use homie_llm::{
    CredentialDestination, CredentialPropagationPolicy, InMemoryVirtualKeyStore,
    ManagedLlmProxyConfig, VirtualKeyError, VirtualKeyRequest, VirtualKeyScope,
};
use homie_proto::{AgentProfileId, ProviderId, SessionId};
use time::{Duration, OffsetDateTime};

#[test]
fn issued_virtual_key_validates_only_for_matching_scope() {
    let mut store = InMemoryVirtualKeyStore::default();
    let scope = key_scope("session_1", "agent_1", "provider_1", &["model-a"]);
    let issued = store.issue(VirtualKeyRequest {
        scope: scope.clone(),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    });

    let claims = store
        .validate(issued.secret.as_str(), &scope, "model-a")
        .expect("valid key");
    assert_eq!(claims.key_id, issued.key_id);

    let wrong_scope = key_scope("session_2", "agent_1", "provider_1", &["model-a"]);
    let error = store
        .validate(issued.secret.as_str(), &wrong_scope, "model-a")
        .expect_err("wrong session should fail");
    assert_eq!(error, VirtualKeyError::ScopeMismatch);
}

#[test]
fn revoked_expired_and_unknown_virtual_keys_are_rejected() {
    let mut store = InMemoryVirtualKeyStore::default();
    let scope = key_scope("session_1", "agent_1", "provider_1", &["model-a"]);
    let expired = store.issue(VirtualKeyRequest {
        scope: scope.clone(),
        expires_at: OffsetDateTime::now_utc() - Duration::seconds(1),
    });
    assert_eq!(
        store.validate(expired.secret.as_str(), &scope, "model-a"),
        Err(VirtualKeyError::Expired)
    );

    let issued = store.issue(VirtualKeyRequest {
        scope: scope.clone(),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    });
    store.revoke(&issued.key_id).expect("revoke");
    assert_eq!(
        store.validate(issued.secret.as_str(), &scope, "model-a"),
        Err(VirtualKeyError::Revoked)
    );

    assert_eq!(
        store.validate("hv_unknown", &scope, "model-a"),
        Err(VirtualKeyError::NotFound)
    );

    let rendered = store
        .validate(issued.secret.as_str(), &scope, "model-a")
        .expect_err("revoked key should fail")
        .to_string();
    assert!(!rendered.contains(issued.secret.as_str()));
}

#[test]
fn scope_denied_covers_profile_provider_and_model() {
    let mut store = InMemoryVirtualKeyStore::default();
    let scope = key_scope("session_1", "agent_1", "provider_1", &["model-a"]);
    let issued = store.issue(VirtualKeyRequest {
        scope: scope.clone(),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    });

    let wrong_profile = key_scope("session_1", "agent_2", "provider_1", &["model-a"]);
    assert_eq!(
        store.validate(issued.secret.as_str(), &wrong_profile, "model-a"),
        Err(VirtualKeyError::ScopeMismatch)
    );

    let wrong_provider = key_scope("session_1", "agent_1", "provider_2", &["model-a"]);
    assert_eq!(
        store.validate(issued.secret.as_str(), &wrong_provider, "model-a"),
        Err(VirtualKeyError::ScopeMismatch)
    );

    assert_eq!(
        store.validate(issued.secret.as_str(), &scope, "model-b"),
        Err(VirtualKeyError::ModelNotAllowed)
    );
}

#[test]
fn managed_proxy_config_serializes_without_raw_provider_key() {
    let mut store = InMemoryVirtualKeyStore::default();
    let scope = key_scope("session_1", "agent_1", "provider_1", &["model-a"]);
    let issued = store.issue(VirtualKeyRequest {
        scope,
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    });
    let issued_debug = format!("{issued:?}");
    let raw_provider_key = "raw-provider-key-test-only";

    assert!(!issued_debug.contains(issued.secret.as_str()));

    let config =
        ManagedLlmProxyConfig::from_issued_key("http://127.0.0.1:4040/v1".to_string(), issued);
    let serialized = serde_json::to_string(&config).expect("serializes");
    let debug = format!("{config:?}");

    assert!(serialized.contains("http://127.0.0.1:4040/v1"));
    assert!(serialized.contains("hv_"));
    assert!(!serialized.contains(raw_provider_key));
    assert!(!serialized.contains("Authorization"));
    assert!(!serialized.contains("secretRef"));
    assert!(!serialized.contains("providerApiKey"));
    assert!(!debug.contains(config.virtual_key.as_str()));
}

#[test]
fn raw_provider_key_is_rejected_for_cross_module_destinations() {
    let raw_provider_key = "raw-provider-key-test-only";
    let policy = CredentialPropagationPolicy::new();

    for destination in [
        CredentialDestination::RemoteNode,
        CredentialDestination::McpTool,
        CredentialDestination::ManagedAgentConfig,
        CredentialDestination::LogEvent,
    ] {
        let error = policy
            .ensure_payload_is_secretless(
                destination,
                format!("provider_key={raw_provider_key}"),
                raw_provider_key,
            )
            .expect_err("raw provider key must be rejected");
        assert_eq!(error, VirtualKeyError::RawProviderKeyForbidden(destination));
        assert!(!error.to_string().contains(raw_provider_key));
    }

    policy
        .ensure_payload_is_secretless(
            CredentialDestination::ManagedAgentConfig,
            "base_url=http://127.0.0.1:4040/v1 virtual_key=hv_test",
            raw_provider_key,
        )
        .expect("virtual key proxy config is allowed");
}

fn key_scope(
    session_id: &str,
    agent_profile_id: &str,
    provider_id: &str,
    models: &[&str],
) -> VirtualKeyScope {
    VirtualKeyScope {
        session_id: SessionId::from(session_id),
        agent_profile_id: AgentProfileId::from(agent_profile_id),
        provider_id: ProviderId::from(provider_id),
        allowed_models: models.iter().map(|model| (*model).to_string()).collect(),
    }
}
