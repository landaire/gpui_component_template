use gpui::ClickEvent;
use gpui::Entity;
use gpui::SharedString;
use gpui::Window;
use gpui::div;
use gpui::prelude::*;
use gpui_component::ActiveTheme;
use gpui_component::IconName;
use gpui_component::Sizable;
use gpui_component::Theme;
use gpui_component::ThemeMode;
use gpui_component::TitleBar;
use gpui_component::button::Button;
use gpui_component::button::ButtonVariants;

use crate::themes::ThemeSwitcher;

pub struct AppTitleBar {
    title: SharedString,
    theme_switcher: Entity<ThemeSwitcher>,
}

impl AppTitleBar {
    pub fn new(title: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let theme_switcher = cx.new(|cx| ThemeSwitcher::new(cx));

        Self {
            title: title.into(),
            theme_switcher,
        }
    }

    fn change_color_mode(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let mode = match cx.theme().mode.is_dark() {
            true => ThemeMode::Light,
            false => ThemeMode::Dark,
        };

        Theme::change(mode, None, cx);
    }
}

impl Render for AppTitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            // Left side
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.title.clone()),
            )
            // Right side
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(self.theme_switcher.clone())
                    .child(
                        Button::new("theme-mode")
                            .map(|this| {
                                if cx.theme().mode.is_dark() {
                                    this.icon(IconName::Sun)
                                } else {
                                    this.icon(IconName::Moon)
                                }
                            })
                            .small()
                            .ghost()
                            .on_click(cx.listener(Self::change_color_mode)),
                    ),
            )
    }
}
