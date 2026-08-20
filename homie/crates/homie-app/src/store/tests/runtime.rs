#[test]
fn inert_runtime_has_no_background_tasks_or_live_sessions() {
    let runtime = super::super::StoreRuntime::inert();
    assert!(
        runtime
            .tasks
            .lock()
            .expect("runtime task lock poisoned")
            .is_empty()
    );
    assert!(runtime.snapshots().borrow().sessions.is_empty());
}
