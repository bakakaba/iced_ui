//! Snapshot tests for the [`iced_ui::NavigationDrawer`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::navigation_drawer::{DrawerItem, NavigationDrawer};
use iced_ui_tests::{Element, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Dismiss,
    Selected(usize),
}

#[test]
fn navigation_drawer_collapsed() -> Result<(), Error> {
    // When collapsed, the drawer should not render its overlay; only
    // the host content is visible.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let element = NavigationDrawer::new(host)
        .push(DrawerItem::header("Navigation"))
        .push(DrawerItem::destination("Home"))
        .push(DrawerItem::destination("Profile"))
        .push(DrawerItem::destination("Settings"))
        .push(DrawerItem::divider())
        .push(DrawerItem::destination("Help"))
        .active(0)
        .modal(true)
        .expanded(false)
        .on_dismiss(Message::Dismiss)
        .on_select(Message::Selected);

    assert_snapshot::<Message>("navigation_drawer_collapsed", element, TALL_SIZE)
}
