use iced::widget::{column, row, text};
use iced_ui::number_input::NumberInput;
use iced_ui::text::Text;
use iced_ui::text_input::TextInput;
use iced_ui::text_input::Variant;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Text(String),
    Search(String),
    Password(String),
    Float(f32),
    Integer(i32),
}

pub(crate) struct InputsPage {
    text_value: String,
    search_value: String,
    password_value: String,
    float_value: f32,
    integer_value: i32,
}

impl Default for InputsPage {
    fn default() -> Self {
        Self {
            text_value: String::new(),
            search_value: String::new(),
            password_value: String::new(),
            float_value: 16.0,
            integer_value: 42,
        }
    }
}

impl super::PageView for InputsPage {
    type Msg = Msg;
    const LABEL: &'static str = "Inputs";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Text(value) => self.text_value = value,
            Msg::Search(value) => self.search_value = value,
            Msg::Password(value) => self.password_value = value,
            Msg::Float(value) => self.float_value = value,
            Msg::Integer(value) => self.integer_value = value,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let outlined_input = TextInput::new("Enter text...", &self.text_value).on_input(Msg::Text);

        let filled_input = TextInput::new("Filled variant", &self.search_value)
            .on_input(Msg::Search)
            .variant(Variant::Filled);

        let password_input = TextInput::new("Password", &self.password_value)
            .on_input(Msg::Password)
            .secure(true);

        let float_input = NumberInput::new(self.float_value)
            .on_change(Msg::Float)
            .range(0.0..=100.0)
            .step(0.5)
            .precision(1);

        let integer_input = NumberInput::new(self.integer_value)
            .on_change(Msg::Integer)
            .range(0..=999)
            .step(1)
            .stepper(false)
            .width(iced::Length::Fixed(100.0));

        column![
            text("Text and numeric input fields.").size(14),
            Text::h2("Text Input"),
            row![outlined_input].width(iced::Length::Fixed(300.0)),
            row![filled_input].width(iced::Length::Fixed(300.0)),
            row![password_input].width(iced::Length::Fixed(300.0)),
            Text::h2("Number Input"),
            row![
                column![
                    text("Float (with stepper)").size(12),
                    float_input,
                    text(format!("Value: {:.1}", self.float_value)).size(12),
                ]
                .spacing(4),
                column![
                    text("Integer (no stepper)").size(12),
                    integer_input,
                    text(format!("Value: {}", self.integer_value)).size(12),
                ]
                .spacing(4),
            ]
            .spacing(24),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
