//! Snapshot tests for the [`iced_ui::Checkbox`] widget.

use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::Checkbox;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    #[allow(dead_code)]
    BoolToggled(bool),
    #[allow(dead_code)]
    OptToggled(Option<bool>),
}

#[test]
fn checkbox_unchecked() -> Result<(), Error> {
    let element = row![Checkbox::new(false).on_toggle(Message::BoolToggled)].padding(20);
    assert_snapshot::<Message>("checkbox_unchecked", element, DEFAULT_SIZE)
}

#[test]
fn checkbox_checked() -> Result<(), Error> {
    let element = row![Checkbox::new(true).on_toggle(Message::BoolToggled)].padding(20);
    assert_snapshot::<Message>("checkbox_checked", element, DEFAULT_SIZE)
}

#[test]
fn checkbox_indeterminate() -> Result<(), Error> {
    let element = row![Checkbox::new(None).on_toggle(Message::OptToggled)].padding(20);
    assert_snapshot::<Message>("checkbox_indeterminate", element, DEFAULT_SIZE)
}

#[test]
fn checkbox_with_label() -> Result<(), Error> {
    let element = row![
        Checkbox::new(true)
            .label(text("Remember me").size(14))
            .on_toggle(Message::BoolToggled)
    ]
    .padding(20);
    assert_snapshot::<Message>("checkbox_with_label", element, DEFAULT_SIZE)
}

#[test]
fn checkbox_disabled() -> Result<(), Error> {
    // No `on_toggle` makes the checkbox non-interactive, rendering the
    // disabled style.
    let element = row![Checkbox::new(true).label(text("Disabled").size(14))].padding(20);
    assert_snapshot::<Message>("checkbox_disabled", element, DEFAULT_SIZE)
}
