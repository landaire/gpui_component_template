use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use gpui::prelude::*;
use gpui::*;
use gpui_component::Root;
use gpui_component::StyledExt;
use gpui_component::Theme;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants as _;
use gpui_component::dock::DockArea;
use gpui_component::dock::DockAreaState;
use gpui_component::dock::DockEvent;
use gpui_component::dock::DockItem;
use gpui_component::dock::Panel;
use gpui_component::dock::PanelEvent;
use gpui_component::h_flex;
use gpui_component::v_flex;
use serde::Deserialize;
use serde::Serialize;

mod second_tab;
mod themes;
mod title_bar;

use second_tab::SecondPanel;

#[derive(Action, Clone, PartialEq)]
#[action(namespace = gpui_component_template, no_json)]
struct SwitchToLight;

#[derive(Action, Clone, PartialEq)]
#[action(namespace = gpui_component_template, no_json)]
struct SwitchToDark;

#[derive(Action, Clone, PartialEq)]
#[action(namespace = gpui_component_template, no_json)]
struct SwitchToTheme(SharedString);

const MAIN_DOCK_AREA: DockAreaTab = DockAreaTab {
    id: "main-dock",
    version: 1,
};

#[cfg(debug_assertions)]
const STATE_FILE: &str = "target/docks.json";
#[cfg(not(debug_assertions))]
const STATE_FILE: &str = "docks.json";

struct DockAreaTab {
    id: &'static str,
    version: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    theme: Option<SharedString>,
}

impl gpui::Global for AppState {}

impl AppState {
    fn global(cx: &App) -> &Self {
        cx.try_global::<Self>().unwrap_or(&AppState { theme: None })
    }

    fn global_mut(cx: &mut App) -> &mut Self {
        if cx.try_global::<Self>().is_none() {
            cx.set_global(AppState { theme: None });
        }
        cx.global_mut::<Self>()
    }
}

pub fn init(cx: &mut App) {
    cx.set_global(AppState { theme: None });
    cx.set_global(Theme::default());
    themes::init(cx);
    cx.activate(true);
}

pub struct GpuiComponentWorkspace {
    title_bar: Entity<AppTitleBar>,
    dock_area: Entity<DockArea>,
    last_layout_state: Option<DockAreaState>,
    _save_layout_task: Option<Task<()>>,
}

impl GpuiComponentWorkspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let dock_area =
            cx.new(|cx| DockArea::new(MAIN_DOCK_AREA.id, Some(MAIN_DOCK_AREA.version), window, cx));

        Self::setup_default_layout(&dock_area, window, cx);

        cx.subscribe_in(
            &dock_area,
            window,
            |this, dock_area, ev: &DockEvent, window, cx| match ev {
                DockEvent::LayoutChanged => this.save_layout(dock_area, window, cx),
                _ => {}
            },
        )
        .detach();

        cx.on_app_quit({
            let dock_area = dock_area.clone();
            move |_, cx| {
                let state = dock_area.read(cx).dump(cx);
                cx.background_executor().spawn(async move {
                    Self::save_state(&state).unwrap();
                })
            }
        })
        .detach();

        let title_bar = cx.new(|cx| AppTitleBar::new("gpui-component Template", cx));

        Self {
            title_bar,
            dock_area,
            last_layout_state: None,
            _save_layout_task: None,
        }
    }

    fn setup_default_layout(
        dock_area: &Entity<DockArea>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Try with split layout containing tabs - this seems to enable drag/drop in the dock example
        let main_layout = DockItem::split_with_sizes(
            gpui::Axis::Vertical,
            vec![DockItem::tabs(
                vec![
                    Arc::new(cx.new(|cx| MainPanel::new(window, cx))),
                    Arc::new(cx.new(|cx| SecondPanel::new(window, cx))),
                ],
                None,
                &dock_area.downgrade(),
                window,
                cx,
            )],
            vec![None],
            &dock_area.downgrade(),
            window,
            cx,
        );

        dock_area.update(cx, |view, cx| {
            view.set_version(MAIN_DOCK_AREA.version, window, cx);
            view.set_center(main_layout, window, cx);
        });
    }

    fn save_layout(
        &mut self,
        dock_area: &Entity<DockArea>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dock_area = dock_area.clone();
        self._save_layout_task = Some(cx.spawn_in(window, async move |workspace, window| {
            Timer::after(Duration::from_secs(10)).await;

            _ = workspace.update_in(window, move |this, _, cx| {
                let dock_area = dock_area.read(cx);
                let state = dock_area.dump(cx);

                let last_layout_state = this.last_layout_state.clone();
                if Some(&state) == last_layout_state.as_ref() {
                    return;
                }

                Self::save_state(&state).unwrap();
                this.last_layout_state = Some(state);
            });
        }));
    }

    fn save_state(state: &DockAreaState) -> Result<()> {
        println!("Save layout...");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(STATE_FILE, json)?;
        Ok(())
    }

    pub fn new_local(cx: &mut App) -> Task<anyhow::Result<WindowHandle<Root>>> {
        let window_size = size(px(1200.0), px(800.0));
        let window_bounds = Bounds::centered(None, window_size, cx);

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                #[cfg(not(target_os = "linux"))]
                titlebar: Some(gpui_component::TitleBar::title_bar_options()),
                window_min_size: Some(gpui::Size {
                    width: px(640.),
                    height: px(480.),
                }),
                #[cfg(target_os = "linux")]
                window_background: gpui::WindowBackgroundAppearance::Transparent,
                #[cfg(target_os = "linux")]
                window_decorations: Some(gpui::WindowDecorations::Client),
                kind: WindowKind::Normal,
                ..Default::default()
            };

            let window = cx.open_window(options, |window, cx| {
                let workspace = cx.new(|cx| GpuiComponentWorkspace::new(window, cx));
                cx.new(|cx| Root::new(workspace.into(), window, cx))
            })?;

            window
                .update(cx, |_, window, cx| {
                    window.activate_window();
                    window.set_window_title("gpui-component Template");
                    cx.on_release(|_, cx| {
                        cx.quit();
                    })
                    .detach();
                })
                .expect("failed to update window");

            Ok(window)
        })
    }
}

impl Render for GpuiComponentWorkspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let drawer_layer = Root::render_drawer_layer(window, cx);
        let modal_layer = Root::render_modal_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .id("gpui-component-workspace")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(self.title_bar.clone())
            .child(self.dock_area.clone())
            .children(drawer_layer)
            .children(modal_layer)
            .children(notification_layer)
    }
}

impl GpuiComponentWorkspace {}

// Main Panel Component
struct MainPanel {
    view: Entity<MainPanelView>,
}

impl MainPanel {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.new(|cx| MainPanelView::new(cx));
        Self { view }
    }
}

impl Panel for MainPanel {
    fn panel_name(&self) -> &'static str {
        "MainPanel"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Main".into_any_element()
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }

    fn zoomable(&self, _cx: &App) -> Option<gpui_component::dock::PanelControl> {
        None
    }
}

impl EventEmitter<PanelEvent> for MainPanel {}

impl Focusable for MainPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.view.focus_handle(cx)
    }
}

impl Render for MainPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.view.clone()
    }
}

struct MainPanelView {
    focus_handle: FocusHandle,
}

impl MainPanelView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for MainPanelView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MainPanelView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_6()
            .p_6()
            .child(div().text_2xl().font_bold().child("Hello World"))
            .child(
                v_flex()
                    .gap_4()
                    .child(div().text_lg().font_semibold().child("Button Examples"))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .child(
                                Button::new("primary-btn")
                                    .primary()
                                    .label("Primary")
                                    .on_click(|_, _, _| println!("Primary clicked")),
                            )
                            .child(
                                Button::new("secondary-btn")
                                    .label("Secondary")
                                    .on_click(|_, _, _| println!("Secondary clicked")),
                            )
                            .child(
                                Button::new("danger-btn")
                                    .danger()
                                    .label("Danger")
                                    .on_click(|_, _, _| println!("Danger clicked")),
                            )
                            .child(
                                Button::new("success-btn")
                                    .success()
                                    .label("Success")
                                    .on_click(|_, _, _| println!("Success clicked")),
                            )
                            .child(
                                Button::new("warning-btn")
                                    .warning()
                                    .label("Warning")
                                    .on_click(|_, _, _| println!("Warning clicked")),
                            )
                            .child(
                                Button::new("info-btn")
                                    .info()
                                    .label("Info")
                                    .on_click(|_, _, _| println!("Info clicked")),
                            )
                            .child(
                                Button::new("ghost-btn")
                                    .ghost()
                                    .label("Ghost")
                                    .on_click(|_, _, _| println!("Ghost clicked")),
                            ),
                    ),
            )
    }
}

use rust_embed::RustEmbed;

use crate::title_bar::AppTitleBar;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/*.svg"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }
        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow::anyhow!("could not find asset at path \"{}\"", path))
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<gpui::SharedString>> {
        Ok(Self::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| p.into())
            .collect())
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        init(cx);

        GpuiComponentWorkspace::new_local(cx).detach();
    });
}
