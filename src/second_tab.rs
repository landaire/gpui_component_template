use gpui::*;
use gpui_component::StyledExt;
use gpui_component::dock::Panel;
use gpui_component::dock::PanelEvent;
use gpui_component::v_flex;

// Second Panel Component
pub struct SecondPanel {
    view: Entity<SecondPanelView>,
}

impl SecondPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.new(|cx| SecondPanelView::new(cx));
        Self { view }
    }
}

impl Panel for SecondPanel {
    fn panel_name(&self) -> &'static str {
        "SecondPanel"
    }

    fn title(&self, _window: &Window, _cx: &App) -> AnyElement {
        "Second Tab".into_any_element()
    }

    fn closable(&self, _cx: &App) -> bool {
        true
    }

    fn zoomable(&self, _cx: &App) -> Option<gpui_component::dock::PanelControl> {
        None
    }
}

impl EventEmitter<PanelEvent> for SecondPanel {}

impl Focusable for SecondPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.view.focus_handle(cx)
    }
}

impl Render for SecondPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.view.clone()
    }
}

struct SecondPanelView {
    focus_handle: FocusHandle,
}

impl SecondPanelView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for SecondPanelView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SecondPanelView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_6()
            .p_6()
            .child(div().text_2xl().font_bold().child("Second Tab"))
            .child(div().text_base().child(
                "This is the content of the second tab. You can drag this tab to create splits!",
            ))
    }
}
