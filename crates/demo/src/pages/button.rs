use iced::widget::{column, row, text};
use iced_ui::button::{Button, ButtonColor, ButtonSize, Variant};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Pressed,
}

#[derive(Default)]
pub(crate) struct ButtonPage;

impl super::PageView for ButtonPage {
    type Msg = Msg;
    const LABEL: &'static str = "Button";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Pressed => super::Action::Log("Button pressed".to_string()),
        }
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let variants_row = row![
            Button::new(text("Solid")).on_press(Msg::Pressed),
            Button::new(text("Outline"))
                .variant(Variant::Outline)
                .on_press(Msg::Pressed),
            Button::new(text("Ghost"))
                .variant(Variant::Ghost)
                .on_press(Msg::Pressed),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let disabled_row = row![
            Button::new(text("Solid"))
                .enabled(false)
                .on_press(Msg::Pressed),
            Button::new(text("Outline"))
                .variant(Variant::Outline)
                .enabled(false)
                .on_press(Msg::Pressed),
            Button::new(text("Ghost"))
                .variant(Variant::Ghost)
                .enabled(false)
                .on_press(Msg::Pressed),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let sizes_row = row![
            Button::new(text("SM"))
                .size(ButtonSize::Sm)
                .on_press(Msg::Pressed),
            Button::new(text("MD"))
                .size(ButtonSize::Md)
                .on_press(Msg::Pressed),
            Button::new(text("LG"))
                .size(ButtonSize::Lg)
                .on_press(Msg::Pressed),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let colors_solid = row![
            Button::new(text("Primary"))
                .color(ButtonColor::Primary)
                .on_press(Msg::Pressed),
            Button::new(text("Secondary"))
                .color(ButtonColor::Secondary)
                .on_press(Msg::Pressed),
            Button::new(text("Success"))
                .color(ButtonColor::Success)
                .on_press(Msg::Pressed),
            Button::new(text("Warning"))
                .color(ButtonColor::Warning)
                .on_press(Msg::Pressed),
            Button::new(text("Danger"))
                .color(ButtonColor::Danger)
                .on_press(Msg::Pressed),
            Button::new(text("Info"))
                .color(ButtonColor::Information)
                .on_press(Msg::Pressed),
            Button::new(text("Foreground"))
                .color(ButtonColor::Foreground)
                .on_press(Msg::Pressed),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let colors_ghost = row![
            Button::new(text("Primary"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Primary)
                .on_press(Msg::Pressed),
            Button::new(text("Secondary"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Secondary)
                .on_press(Msg::Pressed),
            Button::new(text("Success"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Success)
                .on_press(Msg::Pressed),
            Button::new(text("Warning"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Warning)
                .on_press(Msg::Pressed),
            Button::new(text("Danger"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Danger)
                .on_press(Msg::Pressed),
            Button::new(text("Info"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Information)
                .on_press(Msg::Pressed),
            Button::new(text("Foreground"))
                .variant(Variant::Ghost)
                .color(ButtonColor::Foreground)
                .on_press(Msg::Pressed),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        column![
            Text::h1("Button"),
            text("Solid, Outline, and Ghost variants with multiple sizes and colors.").size(14),
            Text::h2("Variants"),
            variants_row,
            Text::h3("Disabled"),
            disabled_row,
            Text::h2("Sizes"),
            sizes_row,
            Text::h2("Colors"),
            Text::h3("Solid"),
            colors_solid,
            Text::h3("Ghost"),
            colors_ghost,
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}
