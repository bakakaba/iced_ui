//! Kitchen-sink demo for every `iced_ui` component.
//!
//! Each component page showcases its default appearance without
//! overriding any theme-driven values (padding, spacing, roundness).

mod app;
mod message;
mod pages;
mod state;

use iced_ui::Theme;

/// Convenience alias: every widget in the demo's tree is themed by
/// `iced_ui::Theme`.
type Element<'a, Message> = iced::Element<'a, Message, Theme>;

pub fn main() -> iced::Result {
    use app::Demo;

    iced::application(Demo::default, Demo::update, Demo::view)
        .title("iced_ui demo")
        .subscription(Demo::subscription)
        .theme(Demo::theme)
        .run()
}
