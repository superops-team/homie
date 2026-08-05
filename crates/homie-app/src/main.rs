use gpui::{
    App, Bounds, Context, FontWeight, Render, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, rgba, size,
};
use gpui_platform::application;
use homie_storage::{StorageConfig, open_or_create};
use std::path::PathBuf;

const MIN_WINDOW_WIDTH: f32 = 980.0;
const MIN_WINDOW_HEIGHT: f32 = 620.0;

#[derive(Clone, Debug)]
struct AppState {
    data_dir: PathBuf,
    schema_version: i64,
    default_profile: String,
    session_count: usize,
    storage_ready: bool,
}

struct HomieWorkbench {
    state: AppState,
}

impl HomieWorkbench {
    fn load() -> Self {
        let data_dir = default_data_dir();
        let mut state = AppState {
            data_dir: data_dir.clone(),
            schema_version: 0,
            default_profile: "unavailable".to_string(),
            session_count: 0,
            storage_ready: false,
        };

        if let Ok(storage) = open_or_create(StorageConfig {
            data_dir: data_dir.clone(),
        }) && storage.migrate().is_ok()
            && storage.seed_defaults().is_ok()
            && let Ok(health) = storage.health_check()
        {
            state.schema_version = health.schema_version;
            state.storage_ready = health.foreign_keys && health.journal_mode == "wal";
            state.default_profile = "agent_codex_default".to_string();
            state.session_count = storage
                .list_sessions()
                .map(|sessions| sessions.len())
                .unwrap_or_default();
        }

        Self { state }
    }

    fn status_color(&self) -> gpui::Rgba {
        if self.state.storage_ready {
            rgb(0x34c759)
        } else {
            rgb(0xff453a)
        }
    }

    fn sidebar_row(label: &'static str, value: impl Into<String>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .rounded(px(8.0))
            .px_3()
            .py_2()
            .bg(rgba(0xffffff12))
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgba(0xffffff88))
                    .child(label),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xffffff))
                    .child(value.into()),
            )
    }

    fn capability_card(title: &'static str, body: &'static str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(rgba(0xffffff14))
            .bg(rgba(0xffffff08))
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xffffff))
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(18.0))
                    .text_color(rgba(0xffffffaa))
                    .child(body),
            )
    }
}

impl Render for HomieWorkbench {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let status_label = if self.state.storage_ready {
            "Ready"
        } else {
            "Needs attention"
        };

        div()
            .size_full()
            .bg(rgb(0x12131a))
            .text_color(rgb(0xffffff))
            .flex()
            .child(
                div()
                    .w(px(280.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .bg(rgb(0x1a1c25))
                    .border_r_1()
                    .border_color(rgba(0xffffff12))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .pb_2()
                            .child(
                                div()
                                    .size(px(12.0))
                                    .rounded_full()
                                    .bg(self.status_color()),
                            )
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Homie"),
                            ),
                    )
                    .child(Self::sidebar_row("Runtime", "Codex default"))
                    .child(Self::sidebar_row("Storage", status_label))
                    .child(Self::sidebar_row(
                        "Sessions",
                        self.state.session_count.to_string(),
                    ))
                    .child(Self::sidebar_row(
                        "Default profile",
                        self.state.default_profile.clone(),
                    ))
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(16.0))
                            .text_color(rgba(0xffffff78))
                            .child(self.state.data_dir.display().to_string()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .p_6()
                    .gap_5()
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(26.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Agent control plane"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .text_color(rgba(0xffffffaa))
                                            .child("Local V1 workspace running on SQLite with Codex as the default runtime."),
                                    ),
                            )
                            .child(
                                div()
                                    .rounded(px(999.0))
                                    .px_3()
                                    .py_1()
                                    .bg(rgba(0x34c75922))
                                    .text_color(rgb(0x8ff0a4))
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(format!("schema v{}", self.state.schema_version)),
                            ),
                    )
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_4()
                            .child(Self::capability_card(
                                "Sessions",
                                "Create and inspect local session records. PTY-backed execution is the next runtime slice.",
                            ))
                            .child(Self::capability_card(
                                "LLM traffic",
                                "Provider keys remain outside agent config. Virtual-key proxy and usage metrics are staged in the V1 architecture.",
                            ))
                            .child(Self::capability_card(
                                "Profiles",
                                "Agent profiles bind runtime, LLM profile, skills, MCP configuration, and permissions.",
                            ))
                            .child(Self::capability_card(
                                "Packaging",
                                "This app bundle is a real GPUI application. DMG builds also include the CLI under bin/homie.",
                            )),
                    )
                    .child(
                        div()
                            .flex_1()
                            .rounded(px(14.0))
                            .border_1()
                            .border_color(rgba(0xffffff12))
                            .bg(rgb(0x0d0f14))
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("Next implementation slices"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .line_height(px(20.0))
                                    .text_color(rgba(0xffffffaa))
                                    .child("1. Codex runtime adapter with PTY/session events\n2. OpenAI-compatible LLM proxy with virtual keys\n3. Terminal grid rendering and scrollback\n4. Command palette, quick open, history, worktrees, and native integrations"),
                            ),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(MIN_WINDOW_WIDTH), px(MIN_WINDOW_HEIGHT))),
                window_background: WindowBackgroundAppearance::Opaque,
                app_id: Some("com.superops.homie".to_string()),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HomieWorkbench::load()),
        )
        .expect("failed to open Homie window");
        cx.activate(true);
    });
}

fn default_data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Homie");
    }
    PathBuf::from(".homie")
}
