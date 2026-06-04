use iced::theme::palette::Pair;
use iced::widget::{Space, column, container, row, text};
use iced::{Color, Length};
use iced_ui::Theme;
use iced_ui::color_picker::ColorPicker;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    PickerColorChanged(Color),
}

pub(crate) struct ColorsPage {
    picker_color: Color,
}

impl Default for ColorsPage {
    fn default() -> Self {
        Self {
            picker_color: Color::from_rgb(0.2, 0.6, 1.0),
        }
    }
}

impl super::PageView for ColorsPage {
    type Msg = Msg;
    const LABEL: &'static str = "Colors";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::PickerColorChanged(color) => self.picker_color = color,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        column![
            Text::h1("Color Tokens"),
            text("Extended palette colors derived from the current theme.").size(14),
            Text::h2("Background"),
            background_swatches(),
            Text::h2("Paper"),
            paper_swatches(),
            Text::h2("Primary"),
            trio_swatches(
                |ext| &ext.primary.base,
                |ext| &ext.primary.weak,
                |ext| &ext.primary.strong,
            ),
            Text::h2("Secondary"),
            trio_swatches(
                |ext| &ext.secondary.base,
                |ext| &ext.secondary.weak,
                |ext| &ext.secondary.strong,
            ),
            Text::h2("Success"),
            trio_swatches(
                |ext| &ext.success.base,
                |ext| &ext.success.weak,
                |ext| &ext.success.strong,
            ),
            Text::h2("Warning"),
            trio_swatches(
                |ext| &ext.warning.base,
                |ext| &ext.warning.weak,
                |ext| &ext.warning.strong,
            ),
            Text::h2("Danger"),
            trio_swatches(
                |ext| &ext.danger.base,
                |ext| &ext.danger.weak,
                |ext| &ext.danger.strong,
            ),
            Text::h2("Information"),
            information_swatches(),
            Text::h2("Color Picker"),
            ColorPicker::new(self.picker_color).on_change(Msg::PickerColorChanged),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}

fn background_swatches<'a>() -> Element<'a, Msg> {
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

fn paper_swatches<'a>() -> Element<'a, Msg> {
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
) -> Element<'a, Msg> {
    row![
        swatch("base", move |theme| *base(theme.extended_palette())),
        swatch("weak", move |theme| *weak(theme.extended_palette())),
        swatch("strong", move |theme| *strong(theme.extended_palette())),
    ]
    .spacing(8)
    .wrap()
    .into()
}

fn information_swatches<'a>() -> Element<'a, Msg> {
    row![
        swatch("base", |theme| theme.information.base),
        swatch("weak", |theme| theme.information.weak),
        swatch("strong", |theme| theme.information.strong),
    ]
    .spacing(8)
    .wrap()
    .into()
}

fn swatch<'a>(label: &'a str, pair_fn: impl Fn(&Theme) -> Pair + 'a) -> Element<'a, Msg> {
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
