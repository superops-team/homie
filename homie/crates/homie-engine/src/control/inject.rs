//! Initial-prompt injection for freshly spawned agents.
//!
//! Pure session-input domain extracted from the `session_spawn` /
//! `session_spawn_remote` handlers: auto-accepting Claude's workspace-trust
//! picker and typing an initial prompt once the composer can actually receive
//! input. Nothing here reaches the control transport or `ControlServer`
//! fields — it talks to live sessions only through a short `Registry` lock.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::registry::Registry;

/// Reads one fact about a live session under a short registry lock; `None`
/// once the session is gone. The injection thread must never hold the lock
/// across its sleeps.
fn with_session<T>(
    registry: &Arc<Mutex<Registry>>,
    session_id: &str,
    read: impl FnOnce(&crate::session::Session) -> T,
) -> Option<T> {
    registry
        .lock()
        .ok()
        .and_then(|guard| guard.get(session_id).map(read))
}

/// Handles the only startup prompt Homie can safely pre-authorize: the exact
/// workspace the user just selected for Claude. Current Claude has no launch
/// flag that skips only workspace trust; its documented bypass flag also
/// disables every tool permission and is deliberately not used.
pub(super) fn prepare_agent_input(
    registry: &Arc<Mutex<Registry>>,
    session_id: &str,
    accept_claude_workspace: bool,
    prompt: Option<&str>,
) {
    if accept_claude_workspace {
        accept_claude_workspace_trust(registry, session_id);
    }
    if let Some(prompt) = prompt {
        inject_initial_prompt(registry, session_id, prompt);
    }
}

/// Answers Claude Code's "do you trust this folder?" picker on the user's
/// behalf so a spawn does not stall behind it and swallow the initial prompt.
///
/// This is a deliberate trade: it auto-grants workspace trust for whatever
/// directory the session was pointed at. That is defensible when the user
/// picked the directory in the UI, and weaker when they did not — an
/// orchestrator spawning into a freshly cloned repository gets trust without
/// anyone affirming it. The window is bounded (20s, and it stops at the first
/// non-matching screen), but a session whose own output contains the matched
/// phrases inside that window would also receive the keystroke.
fn accept_claude_workspace_trust(registry: &Arc<Mutex<Registry>>, session_id: &str) {
    for _ in 0..200 {
        let Some((exited, screen)) = with_session(registry, session_id, |session| {
            (session.view().exited, session.screen_lines().join("\n"))
        }) else {
            return;
        };
        if exited {
            return;
        }
        if is_claude_workspace_trust_screen(&screen) {
            let _ = with_session(registry, session_id, |session| session.send_text("1", true));
            // Let Claude persist trust and replace the picker before a caller's
            // initial prompt starts its own readiness/verification loop.
            for _ in 0..20 {
                std::thread::sleep(Duration::from_millis(100));
                let changed = with_session(registry, session_id, |session| {
                    !is_claude_workspace_trust_screen(&session.screen_lines().join("\n"))
                })
                .unwrap_or(true);
                if changed {
                    return;
                }
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub(super) fn is_claude_workspace_trust_screen(screen: &str) -> bool {
    let normalized = screen.to_ascii_lowercase();
    normalized.contains("yes, i trust this folder")
        && (normalized.contains("1.") || normalized.contains("1 "))
}

/// Types an initial prompt into a freshly spawned agent.
///
/// The old shape of this — paste-and-Enter in one go, then call it settled
/// the moment the screen changed at all — lost the prompt outright against
/// Claude Code: its banner and tips repaint for seconds after bracketed-paste
/// mode comes on, so the "screen changed" tell fired on a repaint while the
/// composer had quietly discarded the keystrokes. The user typed a prompt,
/// got a bare agent, and the prompt was gone.
///
/// So the Enter is no longer sent blind, and "it landed" is no longer
/// inferred from the screen merely moving. Each attempt TYPES the prompt
/// without submitting and watches for it to echo into the composer; if it
/// does, the Enter follows a prompt we can see. If it does not — which also
/// describes a line-mode reader that paints nothing before a newline — the
/// Enter goes out anyway and the prompt itself must then appear on screen.
/// Only when neither happens is the attempt treated as swallowed, and only
/// then is anything retyped.
///
/// And it keeps trying. A first-run agent can sit on a trust dialog or a
/// login for a minute before it has a composer at all (Codex asks whether it
/// trusts the directory), which the old three-quick-tries shape treated as
/// "prompt lost". The prompt is held rather than fired: a dialog does not
/// echo what it is handed, so those attempts simply fail their check and come
/// back a moment later, and the first attempt after the dialog closes is the
/// one that lands. Nothing here consults the session's status — Codex reads
/// as `Working` even at an idle composer, so the echo is the only tell worth
/// trusting.
fn inject_initial_prompt(registry: &Arc<Mutex<Registry>>, session_id: &str, prompt: &str) {
    if !wait_until_ready(registry, session_id) {
        return;
    }
    let give_up_at = Instant::now() + PROMPT_INJECTION_WINDOW;
    loop {
        let Some(before) = screen_text(registry, session_id) else {
            return;
        };
        // A word already on screen (a path in the banner, a word from the
        // tips panel) proves nothing, so the probe is chosen against the
        // pre-typing screen.
        let probe = verification_probe(prompt, &before);
        if with_session(registry, session_id, |session| session.paste_text(prompt)).is_none() {
            return;
        }
        match wait_for_echo(registry, session_id, probe.as_deref(), &before, ECHO_WINDOW) {
            EchoOutcome::Gone => return,
            // The composer is holding our text: the Enter is safe.
            EchoOutcome::Visible => {
                submit_typed_prompt(registry, session_id, probe.as_deref());
                return;
            }
            EchoOutcome::Missing => {}
        }

        // Nothing came back. Either the keystrokes were discarded, or this is
        // a reader that paints nothing until it sees a newline (a line-mode
        // shell with echo off). Submitting tells the two apart: the prompt
        // shows up when it landed, and nothing shows up when it did not.
        //
        // This Enter is a keypress into something we cannot see, and some of
        // those things are questions — Codex's "do you trust this directory?"
        // reads Enter as yes. Nothing here can tell a line-mode reader from a
        // dialog: both swallow a paste without repainting, and the difference
        // is canonical vs raw mode, which lives in the holder's pty and not
        // here. The old code sent this same blind Enter on every attempt, so
        // the exposure is unchanged; narrowing it would need the holder to
        // report termios.
        if with_session(registry, session_id, |session| session.submit_input()).is_none() {
            return;
        }
        match wait_for_echo(
            registry,
            session_id,
            probe.as_deref(),
            &before,
            LANDED_WINDOW,
        ) {
            EchoOutcome::Gone | EchoOutcome::Visible => return,
            EchoOutcome::Missing => {}
        }

        // Truly swallowed. Empty the composer before retyping so a late echo
        // cannot concatenate with the retry.
        if with_session(registry, session_id, |session| session.clear_input_line()).is_none() {
            return;
        }
        if !sleep_until(give_up_at, PROMPT_RETRY_DELAY) {
            break;
        }
    }
    eprintln!(
        "homied: {session_id} never accepted its initial prompt within \
         {}s — left untyped rather than submitted blind",
        PROMPT_INJECTION_WINDOW.as_secs()
    );
}

/// How long a prompt waits for a composer that will take it. Long enough to
/// outlast a trust dialog or a first-run login, short enough that a session
/// abandoned at a wall does not hold a thread forever.
const PROMPT_INJECTION_WINDOW: Duration = Duration::from_secs(180);

/// Quiet time between delivery attempts.
const PROMPT_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Sleeps for `delay`, or reports false when that would pass `deadline`.
fn sleep_until(deadline: Instant, delay: Duration) -> bool {
    if Instant::now() + delay >= deadline {
        return false;
    }
    std::thread::sleep(delay);
    true
}

/// What the screen said about a prompt we just typed.
enum EchoOutcome {
    /// The prompt is visibly sitting in the composer: safe to submit.
    Visible,
    /// Nothing arrived; the composer can be cleared and the prompt retyped.
    Missing,
    /// The session exited or vanished — stop touching it.
    Gone,
}

/// How long to watch for the prompt to echo back as it is typed, and how long
/// to watch for it after submitting. The first is short because a TUI that
/// renders its composer does so immediately; the second is longer because it
/// covers a round trip through the agent.
const ECHO_WINDOW: Duration = Duration::from_millis(1500);
const LANDED_WINDOW: Duration = Duration::from_millis(2500);

/// Polls for the typed prompt to appear on screen. With no usable probe —
/// every word of the prompt was already on screen — any change from `before`
/// is taken as the echo, which is the best signal available in that case.
fn wait_for_echo(
    registry: &Arc<Mutex<Registry>>,
    session_id: &str,
    probe: Option<&str>,
    before: &str,
    window: Duration,
) -> EchoOutcome {
    let polls = (window.as_millis() / 100).max(1);
    for _ in 0..polls {
        std::thread::sleep(Duration::from_millis(100));
        let Some((exited, now)) = with_session(registry, session_id, |session| {
            (session.view().exited, session.screen_lines().join("\n"))
        }) else {
            return EchoOutcome::Gone;
        };
        if exited {
            return EchoOutcome::Gone;
        }
        let echoed = probe.map_or_else(|| now != before, |probe| now.contains(probe));
        if echoed {
            return EchoOutcome::Visible;
        }
    }
    EchoOutcome::Missing
}

/// Presses Enter on a prompt already verified to be in the composer, and
/// confirms the composer let go of it. A prompt still sitting there after the
/// first Enter gets exactly one more — never a retype, which is what would
/// double-send.
fn submit_typed_prompt(registry: &Arc<Mutex<Registry>>, session_id: &str, probe: Option<&str>) {
    for _ in 0..2 {
        if with_session(registry, session_id, |session| session.submit_input()).is_none() {
            return;
        }
        let Some(probe) = probe else {
            return;
        };
        // Submitting moves the prompt out of the composer and into the
        // transcript above it; either way the agent now owns it. Only a
        // screen that never moved at all means the Enter was swallowed.
        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(100));
            match screen_text(registry, session_id) {
                None => return,
                Some(now)
                    if !now.contains(probe) || agent_started_working(registry, session_id) =>
                {
                    return;
                }
                Some(_) => {}
            }
        }
    }
}

/// True once the session's own status reducer says the agent is doing
/// something — the prompt was received even if its text is still echoed in
/// the transcript above the composer.
fn agent_started_working(registry: &Arc<Mutex<Registry>>, session_id: &str) -> bool {
    with_session(registry, session_id, |session| {
        matches!(
            session.view().status,
            homie_proto::SessionStatus::Working | homie_proto::SessionStatus::NeedsInput(_)
        )
    })
    .unwrap_or(false)
}

fn screen_text(registry: &Arc<Mutex<Registry>>, session_id: &str) -> Option<String> {
    with_session(registry, session_id, |session| {
        (!session.view().exited).then(|| session.screen_lines().join("\n"))
    })
    .flatten()
}

/// Waits until the agent can actually receive typed input. First for the
/// exec (a deferred launch fires within its fallback window), then for the
/// input line to come alive — bracketed-paste mode is the tell across
/// Claude/Codex/Cursor/Gemini. Falls back to "screen non-blank and settled"
/// for agents that never enable paste mode, and hard-caps the wait. False
/// means stop: the session exited or vanished.
fn wait_until_ready(registry: &Arc<Mutex<Registry>>, session_id: &str) -> bool {
    for _ in 0..40 {
        // ≤ ~4s for the PTY to be spawned (deferred launch included).
        match with_session(registry, session_id, |session| {
            (session.view().exited, session.child_pid())
        }) {
            None | Some((true, _)) => return false,
            Some((false, pid)) if pid > 0 => break,
            Some(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
    let mut last_text = String::new();
    let mut stable_ticks = 0;
    for tick in 0..200 {
        // ≤ ~20s hard cap; Claude's first paint can be slow.
        let Some((exited, paste, text)) = with_session(registry, session_id, |session| {
            (
                session.view().exited,
                session.bracketed_paste(),
                session.screen_lines().join("\n"),
            )
        }) else {
            return false;
        };
        if exited {
            return false;
        }
        if paste {
            // Paste mode says the input line exists; it does NOT say the TUI
            // has stopped repainting over it. Claude Code turns paste mode on
            // while its banner and tips panel are still landing, and anything
            // typed into that window is discarded. Wait for the screen to
            // hold still before treating the composer as real.
            return screen_settled(registry, session_id);
        }
        if !text.trim().is_empty() && text == last_text {
            stable_ticks += 1;
            if stable_ticks >= 6 && tick >= 10 {
                return true; // ~600ms stable, at least ~1s in
            }
        } else {
            stable_ticks = 0;
            last_text = text;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    true
}

/// Waits (≤ ~5s) for the screen to stop changing, so the prompt is typed into
/// a composer that has finished being drawn over. True unless the session
/// exited or vanished; a TUI that simply never goes quiet (an animated
/// spinner in the banner) still gets its prompt, verified by the echo.
fn screen_settled(registry: &Arc<Mutex<Registry>>, session_id: &str) -> bool {
    let mut last = String::new();
    let mut stable_ticks = 0;
    for _ in 0..50 {
        let Some((exited, text)) = with_session(registry, session_id, |session| {
            (session.view().exited, session.screen_lines().join("\n"))
        }) else {
            return false;
        };
        if exited {
            return false;
        }
        if text == last {
            stable_ticks += 1;
            if stable_ticks >= 3 {
                return true;
            }
        } else {
            stable_ticks = 0;
            last = text;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    true
}

/// A fragment of the prompt whose presence on screen means the composer
/// received it.
///
/// It has to be a WHOLE word, not a leading slice: composers soft-wrap, and
/// wrapping happens at word boundaries, so any prefix of the prompt can be
/// split across two screen lines while a single word survives intact. It also
/// has to be absent from `before`, or a word the banner already displays
/// would read as an echo the instant we looked. `None` when the prompt offers
/// nothing that qualifies — a prompt made entirely of words already on
/// screen, or of words too long to escape wrapping.
fn verification_probe(prompt: &str, before: &str) -> Option<String> {
    prompt
        .split_whitespace()
        .filter(|word| (MIN_PROBE_CHARS..=MAX_PROBE_CHARS).contains(&word.chars().count()))
        .filter(|word| !before.contains(*word))
        .max_by_key(|word| word.chars().count())
        .map(str::to_owned)
}

/// Short words appear by coincidence; long ones are the ones a narrow
/// composer breaks mid-word.
const MIN_PROBE_CHARS: usize = 4;
const MAX_PROBE_CHARS: usize = 20;
