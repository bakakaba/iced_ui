//! Snapshot tests for the [`iced_ui::Card`] widget.

use iced::Length;
use iced::widget::{column, row, text};
use iced_test::Error;
use iced_ui::card::Card;
use iced_ui_tests::{DEFAULT_SIZE, WIDE_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {}

#[test]
fn card_flat_default() -> Result<(), Error> {
    let element = row![
        Card::new(
            column![
                text("Flat").size(18),
                text("Bordered frame with no shadow.").size(14),
            ]
            .spacing(6),
        )
        .width(Length::Fixed(220.0))
    ]
    .padding(20);

    assert_snapshot::<Message>("card_flat_default", element, DEFAULT_SIZE)
}

#[test]
fn card_elevated() -> Result<(), Error> {
    let element = row![
        Card::new(
            column![
                text("Elevated").size(18),
                text("Drop shadow, no border.").size(14),
            ]
            .spacing(6),
        )
        .width(Length::Fixed(220.0))
        .elevated()
    ]
    .padding(20);

    assert_snapshot::<Message>("card_elevated", element, DEFAULT_SIZE)
}

#[test]
fn card_with_raster_background() -> Result<(), Error> {
    let handle = checker_handle();
    let element = row![
        Card::new(text("Raster image").size(18))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .background_image(handle)
    ]
    .padding(20);

    assert_snapshot::<Message>("card_with_raster_background", element, WIDE_SIZE)
}

#[test]
fn card_with_svg_background() -> Result<(), Error> {
    let handle = gradient_svg_handle();
    let element = row![
        Card::new(text("SVG image").size(18))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .elevated()
            .background_svg(handle)
    ]
    .padding(20);

    assert_snapshot::<Message>("card_with_svg_background", element, WIDE_SIZE)
}

fn checker_handle() -> iced::advanced::image::Handle {
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
    iced::advanced::image::Handle::from_rgba(W, H, pixels)
}

fn gradient_svg_handle() -> iced::advanced::svg::Handle {
    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 120" preserveAspectRatio="none">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#8e2de2"/>
      <stop offset="100%" stop-color="#4a00e0"/>
    </linearGradient>
  </defs>
  <rect width="200" height="120" fill="url(#g)"/>
  <circle cx="160" cy="30" r="22" fill="#ffffff" fill-opacity="0.25"/>
</svg>"##;
    iced::advanced::svg::Handle::from_memory(SVG.to_vec())
}
