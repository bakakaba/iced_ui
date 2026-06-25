use iced::widget::{column, container, row, stack, text};
use iced::{Alignment, Length};
use iced_ui::progress::{self, Dock, Progress};
use iced_ui::spinner::{self, Spinner};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct LoadingPage;

impl super::PageView for LoadingPage {
    type Msg = Msg;
    const LABEL: &'static str = "Loading";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let labeled = |label: &'static str, element: Element<'static, Msg>| -> Element<'_, Msg> {
            column![element, text(label).size(12)]
                .spacing(8)
                .align_x(Alignment::Center)
                .into()
        };

        // Spinner — default, then each color token.
        let spinners = row![
            labeled("Default (Info)", Spinner::new().into()),
            labeled(
                "Primary",
                Spinner::new().color(spinner::Color::Primary).into()
            ),
            labeled(
                "Success",
                Spinner::new().color(spinner::Color::Success).into()
            ),
            labeled(
                "Warning",
                Spinner::new().color(spinner::Color::Warning).into()
            ),
            labeled(
                "Danger",
                Spinner::new().color(spinner::Color::Danger).into()
            ),
        ]
        .spacing(32)
        .align_y(Alignment::Center);

        // Progress — determinate at a sample value.
        let determinate = column![
            track(Progress::determinate(0.0).into()),
            track(Progress::determinate(0.35).into()),
            track(Progress::determinate(0.7).into()),
            track(Progress::determinate(1.0).into()),
        ]
        .spacing(16);

        // Progress — indeterminate, default then each color token.
        let indeterminate = column![
            track(Progress::indeterminate().into()),
            track(
                Progress::indeterminate()
                    .color(progress::Color::Primary)
                    .into()
            ),
            track(
                Progress::indeterminate()
                    .color(progress::Color::Success)
                    .into()
            ),
            track(
                Progress::indeterminate()
                    .color(progress::Color::Warning)
                    .into()
            ),
            track(
                Progress::indeterminate()
                    .color(progress::Color::Danger)
                    .into()
            ),
        ]
        .spacing(16);

        // Progress — docked to the edge of a content panel.
        let docked = row![
            labeled_bar(
                "Top",
                docked_panel(Progress::indeterminate().dock(Dock::Top), Dock::Top),
            ),
            labeled_bar(
                "Bottom",
                docked_panel(Progress::indeterminate().dock(Dock::Bottom), Dock::Bottom),
            ),
        ]
        .spacing(24)
        .wrap();

        column![
            text(
                "Indeterminate spinner and a progress bar with determinate and indeterminate modes."
            )
            .size(14),
            Text::h2("Spinner"),
            text("An indeterminate activity indicator. Color tokens default to Info.").size(14),
            spinners,
            Text::h2("Progress — Determinate"),
            text("Fills a known fraction of the track.").size(14),
            determinate,
            Text::h2("Progress — Indeterminate"),
            text(
                "A segment that loops continuously in one direction. Color tokens default to Info."
            )
            .size(14),
            indeterminate,
            Text::h2("Progress — Docked"),
            text("Docked flush to the edge of a content surface, elevated so it floats above the content.").size(14),
            docked,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}

/// Constrain a full-width progress bar to a readable demo width.
fn track(element: Element<'_, Msg>) -> Element<'_, Msg> {
    container(element).width(Length::Fixed(320.0)).into()
}

/// Pairs a label with a demo element in a centered column.
fn labeled_bar<'a>(label: &'static str, element: Element<'a, Msg>) -> Element<'a, Msg> {
    column![text(label).size(12), element].spacing(8).into()
}

/// Frames a docked progress bar flush against the edge of a content
/// surface so the docking and elevation read as a bar capping a panel.
///
/// The bar is overlaid on top of the panel (via [`stack`]) and aligned
/// to the docked edge, so its drop shadow renders over the content it
/// caps instead of being occluded by the panel surface.
fn docked_panel(progress: Progress<'static>, dock: Dock) -> Element<'static, Msg> {
    let panel = container(text("Content").size(16))
        .padding(16)
        .width(Length::Fixed(280.0))
        .height(Length::Fixed(120.0))
        .style(|theme: &iced_ui::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(
                theme.extended_palette().background.weak.color,
            )),
            ..Default::default()
        });

    // Pin the bar to the docked edge of the panel.
    let edge = match dock {
        Dock::Bottom => Alignment::End,
        // Top (and the inline fallback) cap the top of the panel.
        _ => Alignment::Start,
    };
    let bar = container(progress)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(edge);

    stack![panel, bar].into()
}
