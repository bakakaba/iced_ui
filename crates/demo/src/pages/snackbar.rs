use std::time::Duration;

use iced::Length;
use iced::widget::{center, column, row, text};
use iced_ui::button::Button;
use iced_ui::position::Position;
use iced_ui::screen::Screen;
use iced_ui::snackbar::{Notification, NotificationId, Notifications, Severity, Snackbar};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    // Anchor
    ShowAnchor(Position),
    DismissAnchor(NotificationId),
    // Severity
    ShowSeverity(Severity),
    DismissSeverity(NotificationId),
    // Auto dismiss
    ShowAutoDismiss,
    DismissAuto(NotificationId),
}

pub(crate) struct SnackbarPage {
    anchor_notifications: Notifications,
    anchor_pos: Position,
    severity_notifications: Notifications,
    auto_notifications: Notifications,
}

impl Default for SnackbarPage {
    fn default() -> Self {
        Self {
            anchor_notifications: Notifications::new(),
            anchor_pos: Position::BottomRight,
            severity_notifications: Notifications::new(),
            auto_notifications: Notifications::new(),
        }
    }
}

impl super::PageView for SnackbarPage {
    type Msg = Msg;
    const LABEL: &'static str = "Snackbar";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::ShowAnchor(pos) => {
                self.anchor_pos = pos;
                self.anchor_notifications
                    .push(Notification::new("Anchored here."));
            }
            Msg::DismissAnchor(id) => {
                self.anchor_notifications.dismiss(&id);
            }
            Msg::ShowSeverity(sev) => {
                let msg = match sev {
                    Severity::Neutral => "Notification.",
                    Severity::Information => "This is an informational message.",
                    Severity::Success => "Operation completed successfully.",
                    Severity::Warning => "Please review before continuing.",
                    Severity::Error => "Something went wrong.",
                };
                self.severity_notifications
                    .push(Notification::new(msg).severity(sev));
            }
            Msg::DismissSeverity(id) => {
                self.severity_notifications.dismiss(&id);
            }
            Msg::ShowAutoDismiss => {
                self.auto_notifications.push(
                    Notification::new("Auto-dismisses in 5 seconds.")
                        .auto_dismiss(Duration::from_secs(5)),
                );
            }
            Msg::DismissAuto(id) => {
                self.auto_notifications.dismiss(&id);
            }
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        // -- Anchor section --
        let anchor_btn = |label: &'static str, pos: Position| -> Element<'_, Msg> {
            Button::new(text(label))
                .on_press(Msg::ShowAnchor(pos))
                .width(Length::Fill)
                .into()
        };

        let anchor_buttons: Element<'_, Msg> = column![
            row![
                anchor_btn("Top Left", Position::TopLeft),
                anchor_btn("Top", Position::Top),
                anchor_btn("Top Right", Position::TopRight),
            ]
            .spacing(8),
            row![
                anchor_btn("Bottom Left", Position::BottomLeft),
                anchor_btn("Bottom", Position::Bottom),
                anchor_btn("Bottom Right", Position::BottomRight),
            ]
            .spacing(8),
        ]
        .spacing(8)
        .width(400)
        .into();

        let anchor_snackbar = Snackbar::new(center(anchor_buttons))
            .notifications(&self.anchor_notifications)
            .on_dismiss(Msg::DismissAnchor)
            .anchor(self.anchor_pos);

        // -- Severity section --
        let severity_btn = |label: &'static str, sev: Severity| -> Element<'_, Msg> {
            Button::new(text(label))
                .on_press(Msg::ShowSeverity(sev))
                .into()
        };

        let severity_buttons: Element<'_, Msg> = row![
            severity_btn("Info", Severity::Information),
            severity_btn("Success", Severity::Success),
            severity_btn("Warning", Severity::Warning),
            severity_btn("Error", Severity::Error),
        ]
        .spacing(8)
        .into();

        let severity_snackbar = Snackbar::new(center(severity_buttons))
            .notifications(&self.severity_notifications)
            .on_dismiss(Msg::DismissSeverity);

        // -- Auto dismiss section --
        let auto_host = center(Button::new(text("Show (5s)")).on_press(Msg::ShowAutoDismiss));
        let auto_snackbar = Snackbar::new(auto_host)
            .notifications(&self.auto_notifications)
            .on_dismiss(Msg::DismissAuto);

        column![
            Text::h1("Snackbar"),
            text("Temporary notification bar overlaid on the host content.").size(14),
            Text::h2("Anchor"),
            Screen::new(anchor_snackbar),
            Text::h2("Severity"),
            Screen::new(severity_snackbar),
            Text::h2("Auto Dismiss"),
            Screen::new(auto_snackbar),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
