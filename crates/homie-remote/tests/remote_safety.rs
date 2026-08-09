use homie_remote::{
    HandoffPlan, HostConfigError, HostEntry, HostNodeConfig, is_excluded_from_handoff,
    validate_host,
};

#[test]
fn host_validation_rejects_incomplete_or_malformed_node_config() {
    let mut host = HostEntry {
        id: "forge".to_string(),
        name: Some("Forge".to_string()),
        ssh: "you@forge".to_string(),
        default_cwd: Some("~/code".to_string()),
        node: Some(HostNodeConfig {
            endpoint: "tcp://100.64.0.2:7337".to_string(),
            token_file: "~/.config/homie/forge.token".to_string(),
            node_id: Some("node-forge".to_string()),
        }),
    };
    validate_host(&host).expect("valid host");

    host.node.as_mut().unwrap().token_file.clear();
    assert_eq!(
        validate_host(&host),
        Err(HostConfigError::IncompleteNodeConfig)
    );

    host.node.as_mut().unwrap().token_file = "~/.config/homie/forge.token".to_string();
    host.node.as_mut().unwrap().endpoint = "tcp://missing-port".to_string();
    assert_eq!(
        validate_host(&host),
        Err(HostConfigError::InvalidNodeEndpoint)
    );
}

#[test]
fn handoff_excludes_credentials_and_build_outputs() {
    for excluded in [
        ".git/config",
        ".env",
        ".env.local",
        "provider-auth.json",
        "credentials.json",
        "id_rsa",
        "target/debug/app",
        "node_modules/pkg/index.js",
    ] {
        assert!(is_excluded_from_handoff(excluded), "{excluded}");
    }
    assert!(!is_excluded_from_handoff("src/main.rs"));

    let plan = HandoffPlan::new(vec![
        "src/main.rs".to_string(),
        ".env".to_string(),
        "target/debug/app".to_string(),
    ]);
    assert_eq!(plan.files, vec!["src/main.rs"]);
    assert!(plan.quarantine);
}
