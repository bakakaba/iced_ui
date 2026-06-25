use iced::advanced::image as advanced_image;
use iced::advanced::svg as advanced_svg;
use iced::widget::{Space, column, row, text};
use iced::{Color, Length};
use iced_ui::card::Card;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct CardPage;

impl super::PageView for CardPage {
    type Msg = Msg;
    const LABEL: &'static str = "Card";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let flat_card = Card::new(
            column![
                text("Flat").size(18),
                text("Bordered frame with no shadow.").size(14),
            ]
            .spacing(6),
        )
        .width(Length::Fixed(220.0));

        let elevated_card = Card::new(
            column![
                text("Elevated").size(18),
                text("Drop shadow, no border.").size(14),
            ]
            .spacing(6),
        )
        .width(Length::Fixed(220.0))
        .elevated();

        let raster_card = Card::new(
            column![
                Space::new().height(Length::Fixed(40.0)),
                text("Raster image").size(18).color(Color::WHITE),
                text("Rounded corners clip the image.")
                    .size(14)
                    .color(Color::WHITE),
            ]
            .spacing(6),
        )
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(140.0))
        .background_image(checker_handle());

        let svg_card = Card::new(
            column![
                Space::new().height(Length::Fixed(40.0)),
                text("SVG image").size(18).color(Color::WHITE),
                text("Vector backgrounds supported.")
                    .size(14)
                    .color(Color::WHITE),
            ]
            .spacing(6),
        )
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(140.0))
        .elevated()
        .background_svg(gradient_svg_handle());

        column![
            text("Contained surface for grouping related content.").size(14),
            Text::h2("Variants"),
            row![flat_card, elevated_card].spacing(16).wrap(),
            Text::h2("Backgrounds"),
            row![raster_card, svg_card].spacing(16).wrap(),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}

fn checker_handle() -> advanced_image::Handle {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut pixels = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let on = ((x / 8) + (y / 8)) % 2 == 0;
            let (r, g, b) = if on {
                (0x22u8, 0x5f, 0x8c)
            } else {
                (0x40u8, 0x8f, 0xc2)
            };
            pixels.extend_from_slice(&[r, g, b, 0xff]);
        }
    }
    advanced_image::Handle::from_rgba(W, H, pixels)
}

fn gradient_svg_handle() -> advanced_svg::Handle {
    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 120" preserveAspectRatio="none">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#8e2de2"/>
      <stop offset="100%" stop-color="#4a00e0"/>
    </linearGradient>
  </defs>
  <rect width="200" height="120" fill="url(#g)"/>
  <circle cx="160" cy="30" r="22" fill="#ffffff" fill-opacity="0.25"/>
  <circle cx="40" cy="90" r="14" fill="#ffffff" fill-opacity="0.15"/>
</svg>"##;
    advanced_svg::Handle::from_memory(SVG.to_vec())
}
