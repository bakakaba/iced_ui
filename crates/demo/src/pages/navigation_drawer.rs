use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::navigation_drawer::{DrawerItem, NavigationDrawer};

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Toggle,
    Close,
    ItemSelected(usize),
}

/// Action returned from update for the parent to handle.
pub(super) enum Action {
    None,
    /// An item was selected — parent should log it.
    LogAction(String),
}

#[derive(Debug, Default)]
pub(super) struct NavigationDrawerPage {
    expanded: bool,
}

impl NavigationDrawerPage {
    pub(super) fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Toggle => {
                self.expanded = !self.expanded;
                Action::None
            }
            Message::Close => {
                self.expanded = false;
                Action::None
            }
            Message::ItemSelected(idx) => {
                self.expanded = false;
                Action::LogAction(format!("Drawer item {idx}"))
            }
        }
    }

    pub(super) fn view(&self, log: &ActionLog) -> Element<'_, Message> {
        let host: Element<'_, Message> = column![
            text("Navigation Drawer").size(20),
            text("Side panel with destinations. Modal with scrim.").size(14),
            IconButton::new(text("Toggle Drawer").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Message::Toggle),
            if let Some(last) = &log.last_action {
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
            .expanded(self.expanded)
            .on_dismiss(Message::Close)
            .on_select(Message::ItemSelected)
            .into()
    }
}
