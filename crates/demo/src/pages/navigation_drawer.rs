use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::navigation_drawer::{DrawerItem, NavigationDrawer};
use lucide_icons::Icon;

use crate::Element;
use crate::icons::lucide;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Toggle,
    Close,
    ItemSelected(usize),
}

#[derive(Default)]
pub(crate) struct NavigationDrawerPage {
    expanded: bool,
}

impl super::PageView for NavigationDrawerPage {
    type Msg = Msg;
    const LABEL: &'static str = "NavDrawer";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Toggle => {
                self.expanded = !self.expanded;
                super::Action::None
            }
            Msg::Close => {
                self.expanded = false;
                super::Action::None
            }
            Msg::ItemSelected(idx) => {
                self.expanded = false;
                super::Action::Log(format!("Drawer item {idx}"))
            }
        }
    }

    fn view(&self, log: &ActionLog) -> Element<'_, Msg> {
        let host: Element<'_, Msg> = column![
            text("Navigation Drawer").size(20),
            text("Side panel with destinations. Modal with scrim.").size(14),
            IconButton::new(text("Toggle Drawer").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Msg::Toggle),
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
            .push(DrawerItem::destination("Home").icon(lucide(Icon::Home)))
            .push(DrawerItem::destination("Profile").icon(lucide(Icon::User)))
            .push(DrawerItem::destination("Settings").icon(lucide(Icon::Settings)))
            .push(DrawerItem::divider())
            .push(DrawerItem::destination("Help").icon(lucide(Icon::CircleHelp)))
            .active(0)
            .modal(true)
            .expanded(self.expanded)
            .on_dismiss(Msg::Close)
            .on_select(Msg::ItemSelected)
            .into()
    }
}
