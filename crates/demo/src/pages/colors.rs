use iced::theme::palette::Pair;
use iced::widget::{Space, column, container, row, text};
use iced::{Color, Length};
use iced_ui::Theme;
use iced_ui::color_picker::ColorPicker;

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    PickerColorChanged(Color),
}

#[derive(Debug)]
pub(super) struct ColorsPage {
    picker_color: Color,
}

impl Default for ColorsPage {
    fn default() -> Self {
        Self {
            picker_color: Color::from_rgb(0.2, 0.6, 1.0),
        }
    }
}

impl ColorsPage {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::PickerColorChanged(color) => self.picker_color = color,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        column![
            text("Color Tokens").size(20),
            text("Extended palette colors derived from the current theme.").size(14),
            text("Background").size(16),
            background_swatches(),
            text("Paper").size(16),
            paper_swatches(),
            text("Primary").size(16),
            trio_swatches(
                |ext| &ext.primary.base,
                |ext| &ext.primary.weak,
                |ext| &ext.primary.strong,
            ),
            text("Secondary").size(16),
            trio_swatches(
                |ext| &ext.secondary.base,
                |ext| &ext.secondary.weak,
                |ext| &ext.secondary.strong,
            ),
            text("Success").size(16),
            trio_swatches(
                |ext| &ext.success.base,
                |ext| &ext.success.weak,
                |ext| &ext.success.strong,
            ),
            text("Warning").size(16),
            trio_swatches(
                |ext| &ext.warning.base,
                |ext| &ext.warning.weak,
                |ext| &ext.warning.strong,
            ),
            text("Danger").size(16),
            trio_swatches(
                |ext| &ext.danger.base,
                |ext| &ext.danger.weak,
                |ext| &ext.danger.strong,
            ),
            text("Information").size(16),
            information_swatches(),
            text("Color Picker").size(16),
            ColorPicker::new(self.picker_color).on_change(Message::PickerColorChanged),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}

fn background_swatches<'a>() -> Element<'a, Message> {
    row![
        swatch("base", |theme| theme.extended_palette().background.base),
        swatch("weakest", |theme| theme
            .extended_palette()
            .background
            .weakest),
        swatch("weaker", |theme| theme.extended_palette().background.weaker),
        swatch("weak", |theme| theme.extended_palette().background.weak),
        swatch("neutral", |theme| theme
            .extended_palette()
            .background
            .neutral),
        swatch("strong", |theme| theme.extended_palette().background.strong),
        swatch("stronger", |theme| theme
            .extended_palette()
            .background
            .stronger),
        swatch("strongest", |theme| theme
            .extended_palette()
            .background
            .strongest),
    ]
    .spacing(8)
    .wrap()
    .into()
}

fn paper_swatches<'a>() -> Element<'a, Message> {
    row![
        swatch("base", |theme| theme.paper.base),
        swatch("weakest", |theme| theme.paper.weakest),
        swatch("weaker", |theme| theme.paper.weaker),
        swatch("weak", |theme| theme.paper.weak),
        swatch("neutral", |theme| theme.paper.neutral),
        swatch("strong", |theme| theme.paper.strong),
        swatch("stronger", |theme| theme.paper.stronger),
        swatch("strongest", |theme| theme.paper.strongest),
    ]
    .spacing(8)
    .wrap()
    .into()
}

fn trio_swatches<'a>(
    base: fn(&iced::theme::palette::Extended) -> &iced::theme::palette::Pair,
    weak: fn(&iced::theme::palette::Extended) -> &iced::theme::palette::Pair,
    strong: fn(&iced::theme::palette::Extended) -> &iced::theme::palette::Pair,
) -> Element<'a, Message> {
    row![
        swatch("base", move |theme| *base(theme.extended_palette())),
        swatch("weak", move |theme| *weak(theme.extended_palette())),
        swatch("strong", move |theme| *strong(theme.extended_palette())),
    ]
    .spacing(8)
    .wrap()
    .into()
}

fn information_swatches<'a>() -> Element<'a, Message> {
    row![
        swatch("base", |theme| theme.information.base),
        swatch("weak", |theme| theme.information.weak),
        swatch("strong", |theme| theme.information.strong),
    ]
    .spacing(8)
    .wrap()
    .into()
}

fn swatch<'a>(label: &'a str, pair_fn: impl Fn(&Theme) -> Pair + 'a) -> Element<'a, Message> {
    container(
        container(
            column![
                Space::new().height(Length::Fixed(24.0)),
                text(label).size(12),
            ]
            .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .width(Length::Fixed(80.0))
    .height(Length::Fixed(50.0))
    .style(move |theme: &Theme| {
        let pair = pair_fn(theme);
        iced::widget::container::Style {
            background: Some(iced::Background::Color(pair.color)),
            border: iced::Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(pair.text),
            ..iced::widget::container::Style::default()
        }
    })
    .into()
}
