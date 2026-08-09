use homie_proto::stream::StreamKind;

use crate::dispatcher::request_handlers;

#[must_use]
pub fn method_capabilities() -> Vec<&'static str> {
    request_handlers()
        .iter()
        .map(|handler| handler.method)
        .collect()
}

#[must_use]
pub const fn stream_capabilities() -> [StreamKind; 2] {
    [StreamKind::EventsV1, StreamKind::TerminalV1]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use homie_proto::Method;
    use homie_proto::stream::StreamKind;

    use super::*;
    use crate::dispatcher::{HandlerClass, request_handlers};

    const EXPECTED_METHODS: &[&str] = &[
        Method::STATE_SNAPSHOT,
        Method::EVENTS_WAIT,
        Method::DAEMON_PREPARE_SHUTDOWN,
        Method::DAEMON_SHUTDOWN,
        Method::SESSION_SPAWN,
        Method::SESSION_LIST,
        Method::SESSION_SNAPSHOT,
        Method::SESSION_STATUS,
        Method::SESSION_ARTIFACTS,
        Method::SESSION_PORTS,
        Method::SESSION_SET_PARENT,
        Method::SESSION_LIST_CHILDREN,
        Method::SESSION_PARENT,
        Method::SESSION_HISTORY,
        Method::SESSION_RESUME_FROM_HISTORY,
        Method::SESSION_READ_DIFF,
        Method::SESSION_SEND_TEXT,
        Method::SESSION_RESIZE,
        Method::SESSION_KILL,
        Method::HOST_LOCATE_REPO,
        Method::WORKTREE_LIST,
        Method::WORKTREE_CREATE,
        Method::WORKTREE_REMOVE,
        Method::WORKTREE_OVERVIEW,
        Method::HOOK_REPORT,
    ];

    #[test]
    fn advertised_methods_equal_the_real_handler_registry() {
        let advertised = method_capabilities().into_iter().collect::<BTreeSet<_>>();
        let handlers = request_handlers()
            .iter()
            .map(|handler| handler.method)
            .collect::<BTreeSet<_>>();

        assert_eq!(advertised, handlers);
    }

    #[test]
    fn registry_is_the_exact_prd_method_set() {
        let actual = request_handlers()
            .iter()
            .map(|handler| handler.method)
            .collect::<BTreeSet<_>>();
        let expected = EXPECTED_METHODS.iter().copied().collect::<BTreeSet<_>>();

        assert_eq!(actual, expected);
    }

    #[test]
    fn every_handler_has_an_execution_class() {
        let counts =
            request_handlers()
                .iter()
                .fold((0_usize, 0_usize, 0_usize), |mut counts, handler| {
                    match handler.class {
                        HandlerClass::Actor => counts.0 += 1,
                        HandlerClass::LongRunning => counts.1 += 1,
                        HandlerClass::AsyncWait => counts.2 += 1,
                    }
                    counts
                });

        assert_eq!(counts, (13, 11, 1));
    }

    #[test]
    fn stream_openers_are_exact() {
        assert_eq!(
            stream_capabilities(),
            [StreamKind::EventsV1, StreamKind::TerminalV1]
        );
    }

    #[test]
    fn future_proto_constants_are_not_advertised() {
        let advertised = method_capabilities().into_iter().collect::<BTreeSet<_>>();

        assert!(!advertised.contains(Method::BROWSER_ACT));
        assert!(!advertised.contains(Method::TEST_RUN));
        assert!(!advertised.contains(Method::LLM_PROXY_STATUS));
        assert!(!advertised.contains(Method::TASK_LIST));
        assert!(!advertised.contains(Method::MEMORY_SEARCH));
        assert!(Method::ALL.len() > advertised.len());
    }
}
