use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleIdentity {
    pub bundle_id: String,
    pub team_id: Option<String>,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateCandidate {
    pub bundle: BundleIdentity,
    pub promised_version: String,
    pub codesign_valid: bool,
    pub spctl_accepted: bool,
    pub download_host: String,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum UpdateTrustError {
    #[error("download host is not allowed")]
    HostNotAllowed,
    #[error("candidate bundle id does not match running app")]
    BundleIdMismatch,
    #[error("candidate team id does not match running app")]
    TeamIdMismatch,
    #[error("candidate version does not match feed")]
    VersionMismatch,
    #[error("codesign verification failed")]
    CodesignFailed,
    #[error("system policy assessment failed")]
    SystemPolicyRejected,
    #[error("candidate is not newer than running version")]
    NotNewer,
}

pub fn verify_update_candidate(
    running: &BundleIdentity,
    candidate: &UpdateCandidate,
    allowed_host: &str,
) -> Result<(), UpdateTrustError> {
    if candidate.download_host != allowed_host {
        return Err(UpdateTrustError::HostNotAllowed);
    }
    if candidate.bundle.bundle_id != running.bundle_id {
        return Err(UpdateTrustError::BundleIdMismatch);
    }
    if candidate.bundle.team_id != running.team_id {
        return Err(UpdateTrustError::TeamIdMismatch);
    }
    if candidate.bundle.version != candidate.promised_version {
        return Err(UpdateTrustError::VersionMismatch);
    }
    if !candidate.codesign_valid {
        return Err(UpdateTrustError::CodesignFailed);
    }
    if !candidate.spctl_accepted {
        return Err(UpdateTrustError::SystemPolicyRejected);
    }
    if !version_is_newer(&candidate.bundle.version, &running.version) {
        return Err(UpdateTrustError::NotNewer);
    }
    Ok(())
}

fn version_is_newer(candidate: &str, running: &str) -> bool {
    let parse = |value: &str| {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(candidate) > parse(running)
}
