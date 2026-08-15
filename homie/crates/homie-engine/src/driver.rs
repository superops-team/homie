//! Optional typed control surface for first-class agents.
//!
//! This module is deliberately not a replacement for PTY/holder/session
//! authority. First slice only defines capabilities, stable unsupported errors,
//! and a fake driver used by tests.

use std::sync::Mutex;

use homie_proto::DriverCapabilities;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriverError {
    pub code: &'static str,
    pub message: String,
}

impl DriverError {
    pub fn unsupported(operation: &'static str) -> Self {
        Self {
            code: "unsupported",
            message: format!("{operation} is not supported by this agent driver"),
        }
    }
}

pub type DriverResult<T> = Result<T, DriverError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelOption {
    pub id: String,
    pub label: String,
}

pub trait AgentDriverControl: Send + Sync {
    fn capabilities(&self) -> DriverCapabilities {
        DriverCapabilities::default()
    }

    fn cancel_turn(&self) -> DriverResult<()> {
        Err(DriverError::unsupported("cancel_turn"))
    }

    fn steer_message(&self, _text: &str) -> DriverResult<()> {
        Err(DriverError::unsupported("steer_message"))
    }

    fn respond_permission(&self, _request_id: &str, _option_id: &str) -> DriverResult<()> {
        Err(DriverError::unsupported("respond_permission"))
    }

    fn model_options(&self) -> DriverResult<Vec<ModelOption>> {
        Err(DriverError::unsupported("model_options"))
    }
}

#[derive(Default)]
pub struct UnsupportedDriver;

impl AgentDriverControl for UnsupportedDriver {}

pub const FAKE_DRIVER_ID: &str = "__fake_driver__";

#[derive(Default)]
pub struct FakeDriver {
    steered_lengths: Mutex<Vec<usize>>,
}

impl FakeDriver {
    pub fn steered_lengths(&self) -> Vec<usize> {
        self.steered_lengths
            .lock()
            .expect("steered lengths")
            .clone()
    }
}

impl AgentDriverControl for FakeDriver {
    fn capabilities(&self) -> DriverCapabilities {
        DriverCapabilities {
            prompt: true,
            cancel_turn: true,
            steer_message: true,
            respond_permission: false,
            model_discovery: true,
            native_resume_cursor: true,
            rollback: false,
            fork: false,
            usage_events: false,
            background_work: false,
        }
    }

    fn cancel_turn(&self) -> DriverResult<()> {
        Ok(())
    }

    fn steer_message(&self, text: &str) -> DriverResult<()> {
        self.steered_lengths
            .lock()
            .expect("steered lengths")
            .push(text.len());
        Ok(())
    }

    fn model_options(&self) -> DriverResult<Vec<ModelOption>> {
        Ok(vec![ModelOption {
            id: "fake-model".into(),
            label: "Fake Model".into(),
        }])
    }
}

pub fn capabilities_for_manifest_id(manifest_id: &str) -> DriverCapabilities {
    if manifest_id == FAKE_DRIVER_ID {
        FakeDriver::default().capabilities()
    } else {
        UnsupportedDriver.capabilities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_driver_returns_stable_errors_and_no_capabilities() {
        let driver = UnsupportedDriver;
        assert_eq!(driver.capabilities(), DriverCapabilities::default());
        assert_eq!(driver.cancel_turn().unwrap_err().code, "unsupported");
        assert_eq!(
            driver.steer_message("hello").unwrap_err().message,
            "steer_message is not supported by this agent driver"
        );
        assert_eq!(
            driver.respond_permission("r", "allow").unwrap_err().code,
            "unsupported"
        );
        assert_eq!(driver.model_options().unwrap_err().code, "unsupported");
    }

    #[test]
    fn fake_driver_declares_capabilities_without_storing_prompt_text() {
        let driver = FakeDriver::default();
        let capabilities = driver.capabilities();
        assert!(capabilities.prompt);
        assert!(capabilities.cancel_turn);
        assert!(capabilities.steer_message);
        assert!(capabilities.model_discovery);
        assert!(capabilities.native_resume_cursor);
        assert!(!capabilities.respond_permission);

        let sensitive = "Authorization: bearer secret\nfull user prompt body";
        driver.steer_message(sensitive).expect("steer");
        assert_eq!(driver.steered_lengths(), vec![sensitive.len()]);
        assert_eq!(driver.model_options().expect("models")[0].id, "fake-model");
        driver.cancel_turn().expect("cancel");
    }
}
