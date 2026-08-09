use homie_proto::SessionId;
use homie_task::{TaskError, TaskRecord, TaskStatus};

#[test]
fn task_can_be_claimed_blocked_and_returned() {
    let mut task = TaskRecord::new("task_1", "Implement");
    assert_eq!(task.status, TaskStatus::Open);

    task.claim(SessionId::from("session_1"));
    assert_eq!(task.status, TaskStatus::Claimed);
    assert_eq!(task.claimed_by.as_ref().unwrap().as_str(), "session_1");

    task.block().expect("block claimed task");
    assert_eq!(task.status, TaskStatus::Blocked);

    task.return_task();
    assert_eq!(task.status, TaskStatus::Returned);
    assert!(task.claimed_by.is_none());
}

#[test]
fn unclaimed_task_cannot_be_blocked() {
    let mut task = TaskRecord::new("task_1", "Implement");
    assert_eq!(task.block(), Err(TaskError::NotClaimed));
}
