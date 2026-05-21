use iced::Length;
use iced::widget::{column, row, text};
use iced_ui::badge::Badge;
use iced_ui::card::Card;
use iced_ui::divider::Divider;
use iced_ui::fab::Fab;
use iced_ui::list;
use iced_ui::top_app_bar::TopAppBar;

use crate::Element;

pub(super) fn build<'a>() -> Element<'a, super::Message> {
    // Top app bar
    let nav_icon: Element<'_, super::Message> = text("=").size(20).into();
    let action: Element<'_, super::Message> = text("?").size(16).into();
    let app_bar = TopAppBar::new("My App")
        .navigation_icon(nav_icon)
        .action(action);

    // A card containing a list with badges
    let inbox_item = Badge::count(text("Inbox").size(14), 3);
    let updates_item = Badge::dot(text("Updates").size(14));
    let items_list = list::List::new()
        .push(list::Item::new(inbox_item))
        .push(list::Item::new(updates_item))
        .push(list::Item::new(text("Drafts").size(14)))
        .push(list::Item::new(text("Sent").size(14)));

    let mail_card = Card::new(column![text("Mailbox").size(18), items_list].spacing(8));

    // A FAB
    let fab = Fab::new(text("+").size(24))
        .label(text("Compose").size(16))
        .on_press(super::Message::Noop);

    // Divider between sections
    let divider = Divider::horizontal();

    let screen = Card::new(
        column![
            app_bar,
            divider,
            mail_card,
            row![iced::widget::Space::new().width(Length::Fill), fab].padding(8),
        ]
        .spacing(16),
    );

    column![
        text("Overview").size(20),
        text("A composed layout demonstrating how iced_ui widgets work together.").size(14),
        screen,
    ]
    .spacing(16)
    .padding(20)
    .into()
}
