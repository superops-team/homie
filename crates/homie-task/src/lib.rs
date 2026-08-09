use homie_proto::SessionId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Open,
    Claimed,
    Blocked,
    Completed,
    Returned,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub claimed_by: Option<SessionId>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TaskError {
    #[error("task is not claimed by a session")]
    NotClaimed,
}

impl TaskRecord {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status: TaskStatus::Open,
            claimed_by: None,
        }
    }

    pub fn claim(&mut self, session_id: SessionId) {
        self.status = TaskStatus::Claimed;
        self.claimed_by = Some(session_id);
    }

    pub fn block(&mut self) -> Result<(), TaskError> {
        if self.claimed_by.is_none() {
            return Err(TaskError::NotClaimed);
        }
        self.status = TaskStatus::Blocked;
        Ok(())
    }

    pub fn return_task(&mut self) {
        self.status = TaskStatus::Returned;
        self.claimed_by = None;
    }
}
