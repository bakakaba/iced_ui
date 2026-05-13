use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::navigation_drawer::{DrawerItem, NavigationDrawer};

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let host: Element<'_, Message> = column![
        text("Navigation Drawer").size(20),
        text("Side panel with destinations. Modal with scrim.").size(14),
        IconButton::new(text("Toggle Drawer").size(14))
            .variant(icon_button::Variant::Filled)
            .size(140.0)
            .on_press(Message::ToggleDrawer),
        if let Some(last) = &demo.last_action {
            text(format!("Last: {last}")).size(12)
        } else {
            text("").size(12)
        },
    ]
    .spacing(16)
    .padding(20)
    .into();

    NavigationDrawer::new(host)
        .push(DrawerItem::header("Navigation"))
        .push(DrawerItem::destination("Home"))
        .push(DrawerItem::destination("Profile"))
        .push(DrawerItem::destination("Settings"))
        .push(DrawerItem::divider())
        .push(DrawerItem::destination("Help"))
        .active(0)
        .modal(true)
        .expanded(demo.drawer_expanded)
        .on_dismiss(Message::CloseDrawer)
        .on_select(Message::DrawerItemSelected)
        .into()
}
