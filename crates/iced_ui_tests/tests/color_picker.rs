//! Snapshot tests for the [`iced_ui::ColorPicker`] widget.

use iced::Color;
use iced::widget::column;
use iced_test::Error;
use iced_ui::color_picker::ColorPicker;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Changed(Color),
}

#[test]
fn color_picker_default() -> Result<(), Error> {
    let initial = Color::from_rgb(0.2, 0.6, 0.8);
    let element = column![ColorPicker::new(initial).on_change(Message::Changed)].padding(20);
    assert_snapshot::<Message>("color_picker_default", element, DEFAULT_SIZE)
}
