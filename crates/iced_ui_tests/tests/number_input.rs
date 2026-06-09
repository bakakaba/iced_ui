//! Snapshot tests for the [`iced_ui::NumberInput`] widget.

use iced::widget::row;
use iced_test::Error;
use iced_ui::number_input::NumberInput;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum Message {
    Changed(f32),
    IntChanged(i32),
}

#[test]
fn number_input_float_default() -> Result<(), Error> {
    let element = row![
        NumberInput::new(42.5_f32)
            .on_change(Message::Changed)
            .step(0.5)
            .precision(1)
    ]
    .padding(20);
    assert_snapshot::<Message>("number_input_float_default", element, DEFAULT_SIZE)
}

#[test]
fn number_input_integer_no_stepper() -> Result<(), Error> {
    let element = row![
        NumberInput::new(99_i32)
            .on_change(Message::IntChanged)
            .step(1)
            .stepper(false)
    ]
    .padding(20);
    assert_snapshot::<Message>("number_input_integer_no_stepper", element, DEFAULT_SIZE)
}
