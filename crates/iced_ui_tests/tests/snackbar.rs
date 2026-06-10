//! Snapshot tests for the [`iced_ui::Snackbar`] widget.

use iced::widget::{column, text};
use iced_test::Error;
use iced_ui::Theme;
use iced_ui::snackbar::{Notification, NotificationId, Notifications, Severity, Snackbar};
use iced_ui_tests::{Element, TALL_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Dismiss(NotificationId),
    Action(NotificationId, usize),
}

#[test]
fn snackbar_empty() -> Result<(), Error> {
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let notifications = Notifications::new();

    let element = Snackbar::new(host)
        .notifications(&notifications)
        .on_dismiss(Message::Dismiss);

    assert_snapshot::<Message>("snackbar_empty", element, TALL_SIZE)
}

#[test]
fn snackbar_single() -> Result<(), Error> {
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let mut notifications = Notifications::new();
    notifications.push(Notification::new("Item has been archived.").action("Undo"));

    let element = Snackbar::new(host)
        .notifications(&notifications)
        .on_dismiss(Message::Dismiss)
        .on_action(Message::Action);

    assert_snapshot::<Message>("snackbar_single", element, TALL_SIZE)
}

#[test]
fn snackbar_multiple_preexisting() -> Result<(), Error> {
    // Multiple notifications pushed before the widget is rendered
    // should all appear stacked.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let mut notifications = Notifications::new();
    notifications.push(Notification::new("First notification"));
    notifications.push(Notification::new("Second notification"));
    notifications.push(Notification::new("Third notification"));

    let element = Snackbar::new(host)
        .notifications(&notifications)
        .on_dismiss(Message::Dismiss);

    assert_snapshot::<Message>("snackbar_multiple_preexisting", element, TALL_SIZE)
}

#[test]
fn snackbar_multiple_with_severity() -> Result<(), Error> {
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let mut notifications = Notifications::new();
    notifications.push(Notification::new("Info message").severity(Severity::Information));
    notifications.push(Notification::new("Success message").severity(Severity::Success));
    notifications.push(Notification::new("Error message").severity(Severity::Error));

    let element = Snackbar::new(host)
        .notifications(&notifications)
        .on_dismiss(Message::Dismiss);

    assert_snapshot::<Message>("snackbar_multiple_with_severity", element, TALL_SIZE)
}

#[test]
fn snackbar_dismissed_not_shown() -> Result<(), Error> {
    // Dismissed notifications should not appear in the overlay.
    let host: Element<'_, Message, Theme> =
        column![text("Host content").size(16)].padding(20).into();

    let mut notifications = Notifications::new();
    notifications.push(Notification::new("First (will dismiss)"));
    notifications.push(Notification::new("Second (active)"));
    notifications.push(Notification::new("Third (active)"));

    // Dismiss the first notification.
    let first_id = notifications.all()[0].id.clone();
    notifications.dismiss(&first_id);

    let element = Snackbar::new(host)
        .notifications(&notifications)
        .on_dismiss(Message::Dismiss);

    // Should only show 2 notifications (second and third).
    assert_snapshot::<Message>("snackbar_dismissed_not_shown", element, TALL_SIZE)
}
