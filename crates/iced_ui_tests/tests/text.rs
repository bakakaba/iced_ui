//! Snapshot tests for the [`iced_ui::Text`] heading widget.

use iced::widget::column;
use iced_test::Error;
use iced_ui::Text;
use iced_ui_tests::{DEFAULT_SIZE, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn text_h1_default() -> Result<(), Error> {
    let element = column![Text::h1("Heading 1")].padding(20);
    assert_snapshot::<Message>("text_h1_default", element, DEFAULT_SIZE)
}

#[test]
fn text_all_levels() -> Result<(), Error> {
    let element = column![
        Text::h1("Heading 1"),
        Text::h2("Heading 2"),
        Text::h3("Heading 3"),
        Text::h4("Heading 4"),
        Text::h5("Heading 5"),
    ]
    .spacing(16)
    .padding(20);

    assert_snapshot::<Message>("text_all_levels", element, TALL_SIZE)
}
