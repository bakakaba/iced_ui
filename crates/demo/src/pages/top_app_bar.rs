use iced::widget::{column, text};
use iced_ui::top_app_bar::TopAppBar;

use crate::Element;

pub(super) fn build<'a>() -> Element<'a, super::Message> {
    let nav_icon: Element<'_, super::Message> = text("<").size(20).into();
    let action1: Element<'_, super::Message> = text("S").size(16).into();
    let action2: Element<'_, super::Message> = text("?").size(16).into();

    let app_bar = TopAppBar::new("Page Title")
        .navigation_icon(nav_icon)
        .action(action1)
        .action(action2);

    column![
        text("Top App Bar").size(20),
        text("Title bar with navigation icon, title, and action icons.").size(14),
        app_bar,
    ]
    .spacing(16)
    .padding(20)
    .into()
}
