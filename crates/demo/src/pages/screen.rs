use iced::widget::{column, row, text};
use iced_ui::screen::{Mode, Screen};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct ScreenPage;

impl super::PageView for ScreenPage {
    type Msg = Msg;
    const LABEL: &'static str = "Screen";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let desktop = Screen::new(
            column![text("Desktop 16:9").size(14)]
                .padding(12)
                .spacing(8),
        );

        let mobile_landscape = Screen::new(
            column![text("Mobile Landscape 20:9").size(14)]
                .padding(12)
                .spacing(8),
        )
        .mode(Mode::MobileLandscape);

        let mobile_portrait = Screen::new(
            column![text("Mobile Portrait 9:20").size(14)]
                .padding(12)
                .spacing(8),
        )
        .mode(Mode::MobilePortrait);

        column![
            text("Aspect-ratio container simulating device viewports.").size(14),
            Text::h2("Desktop (16:9)"),
            desktop,
            Text::h2("Mobile"),
            row![mobile_landscape, mobile_portrait].spacing(16),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
