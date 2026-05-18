//! Snapshot tests for the [`iced_ui::TopAppBar`] widget.

use iced::widget::text;
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::TopAppBar;
use iced_ui_tests::{Element, WIDE_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn top_app_bar_default() -> Result<(), Error> {
    let element = TopAppBar::new("Page Title");
    assert_snapshot::<Message>("top_app_bar_default", element, WIDE_SIZE)
}

#[test]
fn top_app_bar_with_actions() -> Result<(), Error> {
    let nav_icon: Element<'_, Message, Theme> = text("<").size(20).into();
    let action1: Element<'_, Message, Theme> = text("S").size(16).into();
    let action2: Element<'_, Message, Theme> = text("?").size(16).into();
    let element = TopAppBar::new("Page Title")
        .navigation_icon(nav_icon)
        .action(action1)
        .action(action2);

    assert_snapshot::<Message>("top_app_bar_with_actions", element, WIDE_SIZE)
}
