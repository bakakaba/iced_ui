use std::time::Duration;

use iced::Length;
use iced::widget::{center, column, row, text};
use iced_ui::button::Button;
use iced_ui::position::Position;
use iced_ui::screen::Screen;
use iced_ui::snackbar::{Severity, Snackbar};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    ShowAnchor(Position),
    HideAnchor,
    ShowSeverity(Severity),
    HideSeverity,
    ShowAutoDismiss,
    HideAutoDismiss,
}

#[derive(Default)]
pub(crate) struct SnackbarPage {
    anchor: Option<Position>,
    severity: Option<Severity>,
    auto_dismiss_visible: bool,
}

impl super::PageView for SnackbarPage {
    type Msg = Msg;
    const LABEL: &'static str = "Snackbar";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::ShowAnchor(pos) => self.anchor = Some(pos),
            Msg::HideAnchor => self.anchor = None,
            Msg::ShowSeverity(sev) => self.severity = Some(sev),
            Msg::HideSeverity => self.severity = None,
            Msg::ShowAutoDismiss => self.auto_dismiss_visible = true,
            Msg::HideAutoDismiss => self.auto_dismiss_visible = false,
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

        let anchor_pos = self.anchor.unwrap_or(Position::BottomRight);
        let anchor_snackbar = Snackbar::new(center(anchor_buttons))
            .message("Anchored here.")
            .on_dismiss(Msg::HideAnchor)
            .anchor(anchor_pos)
            .visible(self.anchor.is_some());

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

        let sev = self.severity.unwrap_or(Severity::Neutral);
        let sev_msg = match sev {
            Severity::Neutral => "Notification.",
            Severity::Information => "This is an informational message.",
            Severity::Success => "Operation completed successfully.",
            Severity::Warning => "Please review before continuing.",
            Severity::Error => "Something went wrong.",
        };
        let severity_snackbar = Snackbar::new(center(severity_buttons))
            .message(sev_msg)
            .on_dismiss(Msg::HideSeverity)
            .severity(sev)
            .visible(self.severity.is_some());

        // -- Auto dismiss section --
        let auto_host = center(Button::new(text("Show (5s)")).on_press(Msg::ShowAutoDismiss));
        let auto_snackbar = Snackbar::new(auto_host)
            .message("Auto-dismisses in 5 seconds.")
            .on_dismiss(Msg::HideAutoDismiss)
            .auto_dismiss(Duration::from_secs(5))
            .visible(self.auto_dismiss_visible);

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
