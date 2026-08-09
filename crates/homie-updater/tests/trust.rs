use homie_updater::{BundleIdentity, UpdateCandidate, UpdateTrustError, verify_update_candidate};

#[test]
fn update_candidate_requires_matching_identity_and_newer_version() {
    let running = running();
    let candidate = UpdateCandidate {
        bundle: BundleIdentity {
            version: "0.2.0".to_string(),
            ..running.clone()
        },
        promised_version: "0.2.0".to_string(),
        codesign_valid: true,
        spctl_accepted: true,
        download_host: "updates.example.invalid".to_string(),
    };
    assert_eq!(
        verify_update_candidate(&running, &candidate, "updates.example.invalid"),
        Ok(())
    );
}

#[test]
fn update_candidate_rejects_wrong_team_or_bundle() {
    let running = running();
    let mut candidate = valid_candidate(&running);
    candidate.bundle.team_id = Some("OTHERTEAM".to_string());
    assert_eq!(
        verify_update_candidate(&running, &candidate, "updates.example.invalid"),
        Err(UpdateTrustError::TeamIdMismatch)
    );

    let mut candidate = valid_candidate(&running);
    candidate.bundle.bundle_id = "com.example.other".to_string();
    assert_eq!(
        verify_update_candidate(&running, &candidate, "updates.example.invalid"),
        Err(UpdateTrustError::BundleIdMismatch)
    );
}

#[test]
fn update_candidate_rejects_policy_or_version_failures() {
    let running = running();
    let mut candidate = valid_candidate(&running);
    candidate.codesign_valid = false;
    assert_eq!(
        verify_update_candidate(&running, &candidate, "updates.example.invalid"),
        Err(UpdateTrustError::CodesignFailed)
    );

    let mut candidate = valid_candidate(&running);
    candidate.spctl_accepted = false;
    assert_eq!(
        verify_update_candidate(&running, &candidate, "updates.example.invalid"),
        Err(UpdateTrustError::SystemPolicyRejected)
    );

    let mut candidate = valid_candidate(&running);
    candidate.bundle.version = "0.1.0".to_string();
    candidate.promised_version = "0.1.0".to_string();
    assert_eq!(
        verify_update_candidate(&running, &candidate, "updates.example.invalid"),
        Err(UpdateTrustError::NotNewer)
    );
}

fn running() -> BundleIdentity {
    BundleIdentity {
        bundle_id: "com.superops.homie".to_string(),
        team_id: Some("TEAMID".to_string()),
        version: "0.1.0".to_string(),
    }
}

fn valid_candidate(running: &BundleIdentity) -> UpdateCandidate {
    UpdateCandidate {
        bundle: BundleIdentity {
            version: "0.2.0".to_string(),
            ..running.clone()
        },
        promised_version: "0.2.0".to_string(),
        codesign_valid: true,
        spctl_accepted: true,
        download_host: "updates.example.invalid".to_string(),
    }
}
