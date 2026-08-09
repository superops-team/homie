#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use homie_remote::RemoteCompanionConfig;
use tempfile::TempDir;

#[test]
fn companion_config_round_trips_with_owner_only_permissions() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("nested/remote.json");
    let config = RemoteCompanionConfig {
        port: 48_620,
        bind_host: Some("100.90.80.70".to_string()),
        token: "example-token".to_string(),
        forward_any_port: Some(false),
    };

    config.save(&path).expect("save");
    assert_eq!(RemoteCompanionConfig::load(&path), Some(config.clone()));

    #[cfg(unix)]
    assert_eq!(
        std::fs::metadata(&path)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn companion_config_remove_is_idempotent() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("remote.json");
    let config = RemoteCompanionConfig {
        port: 48_620,
        bind_host: Some("127.0.0.1".to_string()),
        token: "example-token".to_string(),
        forward_any_port: None,
    };
    config.save(&path).expect("save");

    RemoteCompanionConfig::remove(&path).expect("remove");
    RemoteCompanionConfig::remove(&path).expect("remove missing");

    assert_eq!(RemoteCompanionConfig::load(&path), None);
}

#[test]
fn companion_config_pairing_url_is_explicit_and_debug_redacts_token() {
    let config = RemoteCompanionConfig {
        port: 48_620,
        bind_host: Some("studio.example.ts.net".to_string()),
        token: "example-token".to_string(),
        forward_any_port: None,
    };

    assert_eq!(config.endpoint_label(), "studio.example.ts.net:48620");
    assert_eq!(
        config.pairing_url().as_deref(),
        Some("homie://studio.example.ts.net:48620?token=example-token")
    );
    let debug = format!("{config:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("example-token"));
}
