use iced::Length;
use iced::widget::{center, column, row, text};
use iced_ui::button::Button;
use iced_ui::screen::Screen;
use iced_ui::slide_sheet::{Anchor, SlideSheet};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    ToggleBottom,
    CloseBottom,
    ResizeBottom(f32),
    ToggleTop,
    CloseTop,
    ResizeTop(f32),
    ToggleLeft,
    CloseLeft,
    ResizeLeft(f32),
    ToggleRight,
    CloseRight,
    ResizeRight(f32),
}

#[derive(Default)]
pub(crate) struct SlideSheetPage {
    bottom_expanded: bool,
    bottom_size: f32,
    top_expanded: bool,
    top_size: f32,
    left_expanded: bool,
    left_size: f32,
    right_expanded: bool,
    right_size: f32,
}

impl SlideSheetPage {
    fn size_or_default(val: f32) -> f32 {
        if val == 0.0 { 0.5 } else { val }
    }
}

impl super::PageView for SlideSheetPage {
    type Msg = Msg;
    const LABEL: &'static str = "SlideSheet";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::ToggleBottom => self.bottom_expanded = !self.bottom_expanded,
            Msg::CloseBottom => self.bottom_expanded = false,
            Msg::ResizeBottom(f) => self.bottom_size = f,
            Msg::ToggleTop => self.top_expanded = !self.top_expanded,
            Msg::CloseTop => self.top_expanded = false,
            Msg::ResizeTop(f) => self.top_size = f,
            Msg::ToggleLeft => self.left_expanded = !self.left_expanded,
            Msg::CloseLeft => self.left_expanded = false,
            Msg::ResizeLeft(f) => self.left_size = f,
            Msg::ToggleRight => self.right_expanded = !self.right_expanded,
            Msg::CloseRight => self.right_expanded = false,
            Msg::ResizeRight(f) => self.right_size = f,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let bottom_sheet = SlideSheet::new(
            center(Button::new(text("Toggle")).on_press(Msg::ToggleBottom)),
            "Sheet anchored to the bottom edge.",
        )
        .expanded(self.bottom_expanded)
        .on_dismiss(Msg::CloseBottom)
        .on_resize(Msg::ResizeBottom)
        .size(Self::size_or_default(self.bottom_size));

        let top_sheet = SlideSheet::new(
            center(Button::new(text("Toggle")).on_press(Msg::ToggleTop)),
            "Sheet anchored to the top edge.",
        )
        .anchor(Anchor::Top)
        .expanded(self.top_expanded)
        .on_dismiss(Msg::CloseTop)
        .on_resize(Msg::ResizeTop)
        .size(Self::size_or_default(self.top_size));

        let left_sheet = SlideSheet::new(
            center(Button::new(text("Toggle")).on_press(Msg::ToggleLeft)),
            "Sheet anchored to the left edge.",
        )
        .anchor(Anchor::Left)
        .expanded(self.left_expanded)
        .on_dismiss(Msg::CloseLeft)
        .on_resize(Msg::ResizeLeft)
        .size(Self::size_or_default(self.left_size));

        let right_sheet = SlideSheet::new(
            center(Button::new(text("Toggle")).on_press(Msg::ToggleRight)),
            "Sheet anchored to the right edge.",
        )
        .anchor(Anchor::Right)
        .expanded(self.right_expanded)
        .on_dismiss(Msg::CloseRight)
        .on_resize(Msg::ResizeRight)
        .size(Self::size_or_default(self.right_size));

        column![
            Text::h1("Slide Sheet"),
            text("A panel that slides from any edge. Drag the handle to resize or dismiss.")
                .size(14),
            row![
                column![Text::h2("Bottom"), Screen::new(bottom_sheet)]
                    .spacing(8)
                    .width(Length::Fill),
                column![Text::h2("Top"), Screen::new(top_sheet)]
                    .spacing(8)
                    .width(Length::Fill),
            ]
            .spacing(16),
            row![
                column![Text::h2("Left"), Screen::new(left_sheet)]
                    .spacing(8)
                    .width(Length::Fill),
                column![Text::h2("Right"), Screen::new(right_sheet)]
                    .spacing(8)
                    .width(Length::Fill),
            ]
            .spacing(16),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
