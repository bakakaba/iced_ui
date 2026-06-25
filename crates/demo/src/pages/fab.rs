use iced::widget::{column, row, text};
use iced_ui::fab::{Fab, FabSize};
use iced_ui::icons::{self, Icon};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Pressed,
}

#[derive(Default)]
pub(crate) struct FabPage;

impl super::PageView for FabPage {
    type Msg = Msg;
    const LABEL: &'static str = "FAB";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Pressed => super::Action::Log("FAB pressed".to_string()),
        }
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let small_fab = Fab::new(icons::icon(Icon::Plus).size(18))
            .size(FabSize::Small)
            .on_press(Msg::Pressed);

        let regular_fab = Fab::new(icons::icon(Icon::Plus).size(24)).on_press(Msg::Pressed);

        let large_fab = Fab::new(icons::icon(Icon::Plus).size(36))
            .size(FabSize::Large)
            .on_press(Msg::Pressed);

        let extended_fab = Fab::new(icons::icon(Icon::Plus).size(18))
            .label(text("Create").size(16))
            .on_press(Msg::Pressed);

        let lowered_fab = Fab::new(icons::icon(Icon::Plus).size(24))
            .lowered()
            .on_press(Msg::Pressed);

        column![
            text("Floating Action Button. Small, Regular, Large, Extended, and Lowered variants.")
                .size(14),
            Text::h2("Sizes"),
            row![small_fab, regular_fab, large_fab, extended_fab, lowered_fab].spacing(16),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
