//! Snapshot tests for the [`iced_ui::List`] widget.

use iced::Length;
use iced::widget::text;
use iced_test::Error;
use iced_ui::list;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn list_default() -> Result<(), Error> {
    let element = list::List::new()
        .push(list::Item::new(text("Apple")))
        .push(list::Item::new(text("Banana")))
        .push(list::Item::new(text("Cherry")))
        .push(list::Item::new(text("Dragonfruit")))
        .push(list::Item::new(text("Elderberry")))
        .width(Length::Fixed(200.0));

    assert_snapshot::<Message>("list_default", element, DEFAULT_SIZE)
}
