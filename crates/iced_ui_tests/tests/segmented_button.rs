//! Snapshot tests for the [`iced_ui::SegmentedButton`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::segmented_button::{Segment, SegmentedButton};
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Selected(usize),
}

#[test]
fn segmented_button_default() -> Result<(), Error> {
    let element = column![
        SegmentedButton::new()
            .push(Segment::new(text("Day")), true)
            .push(Segment::new(text("Week")), false)
            .push(Segment::new(text("Month")), false)
            .on_press(Message::Selected)
    ]
    .padding(20);

    assert_snapshot::<Message>("segmented_button_default", element, DEFAULT_SIZE)
}

#[test]
fn segmented_button_middle_selected() -> Result<(), Error> {
    let element = column![
        SegmentedButton::new()
            .push(Segment::new(text("Day")), false)
            .push(Segment::new(text("Week")), true)
            .push(Segment::new(text("Month")), false)
            .on_press(Message::Selected)
    ]
    .padding(20);

    assert_snapshot::<Message>("segmented_button_middle_selected", element, DEFAULT_SIZE)
}
