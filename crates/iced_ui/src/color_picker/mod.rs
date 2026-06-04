//! A color picker widget with a trigger swatch and popup overlay.
//!
//! # Example
//!
//! ```ignore
//! use iced::Color;
//! use iced_ui::color_picker::ColorPicker;
//!
//! let picker = ColorPicker::new(Color::from_rgb(0.2, 0.6, 1.0))
//!     .on_change(Message::ColorChanged);
//! ```

mod style;

pub use style::{Catalog, Status, Style, StyleFn, default};

use std::f32::consts::FRAC_PI_2;

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::keyboard;
use iced::mouse;
use iced::{Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::{FontSizeBase, RoundnessBase, SpacingBase};

// -- HSV color model --

/// Hue-Saturation-Value representation.
#[derive(Debug, Clone, Copy, Default)]
struct Hsva {
    h: f32, // 0.0..=360.0
    s: f32, // 0.0..=1.0
    v: f32, // 0.0..=1.0
}

impl Hsva {
    fn from_color(c: Color) -> Self {
        let r = c.r;
        let g = c.g;
        let b = c.b;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        Self { h, s, v }
    }

    fn to_color(self) -> Color {
        let Hsva { h, s, v } = self;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Color::from_rgb(r + m, g + m, b + m)
    }

    /// Returns the fully-saturated, full-value color for this hue.
    fn hue_color(h: f32) -> Color {
        Self { h, s: 1.0, v: 1.0 }.to_color()
    }

    /// Convert to HSL for display.
    fn to_hsl(self) -> (f32, f32, f32) {
        let l = self.v * (1.0 - self.s / 2.0);
        let s = if l == 0.0 || l == 1.0 {
            0.0
        } else {
            (self.v - l) / l.min(1.0 - l)
        };
        (self.h, s, l)
    }
}

// -- Constants --

/// The width of the popup (SV area + padding).
const POPUP_WIDTH: f32 = 220.0;
/// The height of the hue slider bar.
const BAR_HEIGHT: f32 = 16.0;
/// Gap between sections in the popup.
const GAP: f32 = 8.0;
/// Padding inside the popup panel.
const POPUP_PADDING: f32 = 8.0;
/// Height of the color code row at the bottom.
const CODE_HEIGHT: f32 = 20.0;
/// Default trigger swatch size.
const DEFAULT_TRIGGER_SIZE: f32 = 32.0;

// -- Color format --

/// Which format the color code is displayed in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ColorFormat {
    #[default]
    Hex,
    Rgb,
    Hsl,
    Oklch,
}

impl ColorFormat {
    fn next(self) -> Self {
        match self {
            Self::Hex => Self::Rgb,
            Self::Rgb => Self::Hsl,
            Self::Hsl => Self::Oklch,
            Self::Oklch => Self::Hex,
        }
    }

    fn format(self, hsva: Hsva) -> String {
        let c = hsva.to_color();
        match self {
            Self::Hex => {
                format!(
                    "#{:02X}{:02X}{:02X}",
                    (c.r * 255.0) as u8,
                    (c.g * 255.0) as u8,
                    (c.b * 255.0) as u8,
                )
            }
            Self::Rgb => {
                format!(
                    "rgb({} {} {})",
                    (c.r * 255.0) as u8,
                    (c.g * 255.0) as u8,
                    (c.b * 255.0) as u8,
                )
            }
            Self::Hsl => {
                let (h, s, l) = hsva.to_hsl();
                format!("hsl({:.0} {:.0}% {:.0}%)", h, s * 100.0, l * 100.0)
            }
            Self::Oklch => {
                let (l, ch, h) = srgb_to_oklch(c);
                format!("oklch({:.2}% {:.3} {:.2})", l * 100.0, ch, h)
            }
        }
    }
}

/// Converts an sRGB [`Color`] (gamma-encoded) to OKLCH coordinates.
///
/// Returns `(L, C, H)` where `L` is in `0.0..=1.0`, `C` is the chroma,
/// and `H` is the hue in degrees `0.0..360.0`.
fn srgb_to_oklch(c: Color) -> (f32, f32, f32) {
    fn to_linear(v: f32) -> f32 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    let r = to_linear(c.r);
    let g = to_linear(c.g);
    let b = to_linear(c.b);

    // Linear sRGB to OKLab (Björn Ottosson).
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let lab_l = 0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_;
    let lab_a = 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_;
    let lab_b = 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_;

    let chroma = (lab_a * lab_a + lab_b * lab_b).sqrt();
    let mut hue = lab_b.atan2(lab_a).to_degrees();
    if hue < 0.0 {
        hue += 360.0;
    }

    (lab_l, chroma, hue)
}

// -- Drag target --

#[derive(Debug, Clone, Copy)]
enum DragTarget {
    SaturationValue,
    Hue,
}

// -- Widget state --

#[derive(Debug, Clone, Default)]
struct State {
    open: bool,
    dragging: Option<DragTarget>,
    color_format: ColorFormat,
    /// The live internal HSV color being edited in the overlay.
    /// Initialized from the input color when the popup opens.
    /// Only published back to the app on release/dismiss.
    hsva: Hsva,
}

// -- Widget struct --

/// A color picker widget.
///
/// Renders as a small rounded swatch showing the current color.
/// Clicking the swatch opens a popup overlay with a full
/// saturation/value area, hue slider, alpha slider, and a clickable
/// color code that cycles between hex, RGB, HSL, and OKLCH formats,
/// and a copy button that writes the current code to the clipboard.
pub struct ColorPicker<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    color: Color,
    on_change: Option<Box<dyn Fn(Color) -> Message + 'a>>,
    trigger_size: f32,
    class: Theme::Class<'a>,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> ColorPicker<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a color picker initialized to the given color.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            on_change: None,
            trigger_size: DEFAULT_TRIGGER_SIZE,
            class: Theme::default(),
            _renderer: std::marker::PhantomData,
        }
    }

    /// Sets the callback fired when the color changes.
    pub fn on_change(mut self, f: impl Fn(Color) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Sets the size of the trigger swatch square (default: 32.0).
    pub fn size(mut self, size: f32) -> Self {
        self.trigger_size = size;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

// -- Widget impl (trigger) --

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ColorPicker<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.trigger_size),
            Length::Fixed(self.trigger_size),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.trigger_size, self.trigger_size))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(bounds)
            && !state.open
        {
            state.open = true;
            state.hsva = Hsva::from_color(self.color);
            shell.invalidate_layout();
            shell.request_redraw();
            shell.capture_event();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let status = if state.open {
            Status::Open
        } else {
            Status::Idle
        };
        let style = theme.style(&self.class, status);
        let bounds = layout.bounds();

        // Color fill (use live internal color when popup is open).
        let display_color = if state.open {
            state.hsva.to_color()
        } else {
            self.color
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.trigger_border,
                ..renderer::Quad::default()
            },
            Background::Color(display_color),
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        if !state.open {
            return None;
        }

        let trigger_bounds = layout.bounds();

        Some(overlay::Element::new(Box::new(ColorPickerOverlay {
            on_change: &self.on_change,
            class: &self.class,
            state,
            trigger_bounds,
            _renderer: std::marker::PhantomData,
        })))
    }
}

// -- Overlay --

struct ColorPickerOverlay<'a, 'b, Message, Theme: Catalog, Renderer: renderer::Renderer> {
    on_change: &'b Option<Box<dyn Fn(Color) -> Message + 'a>>,
    class: &'b Theme::Class<'a>,
    state: &'b mut State,
    trigger_bounds: Rectangle,
    _renderer: std::marker::PhantomData<Renderer>,
}

/// Regions within the popup content area (relative to popup inner bounds).
struct PopupRegions {
    sv: Rectangle,
    hue: Rectangle,
    code: Rectangle,
    copy: Rectangle,
}

/// Width of the copy button at the end of the code row.
const COPY_BUTTON_WIDTH: f32 = 24.0;
/// Gap between the code text and the copy button.
const COPY_BUTTON_GAP: f32 = 4.0;

fn popup_inner_height() -> f32 {
    let sv_size = POPUP_WIDTH - POPUP_PADDING * 2.0;
    sv_size + GAP + BAR_HEIGHT + GAP + CODE_HEIGHT
}

fn popup_regions(inner_origin: Point) -> PopupRegions {
    let sv_size = POPUP_WIDTH - POPUP_PADDING * 2.0;
    let w = sv_size;
    let mut y = inner_origin.y;
    let x = inner_origin.x;

    let sv = Rectangle::new(Point::new(x, y), Size::new(sv_size, sv_size));
    y += sv_size + GAP;

    let hue = Rectangle::new(Point::new(x, y), Size::new(w, BAR_HEIGHT));
    y += BAR_HEIGHT + GAP;

    let code_w = w - COPY_BUTTON_WIDTH - COPY_BUTTON_GAP;
    let code = Rectangle::new(Point::new(x, y), Size::new(code_w, CODE_HEIGHT));
    let copy = Rectangle::new(
        Point::new(x + code_w + COPY_BUTTON_GAP, y),
        Size::new(COPY_BUTTON_WIDTH, CODE_HEIGHT),
    );

    PopupRegions {
        sv,
        hue,
        code,
        copy,
    }
}

impl<'a, 'b, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for ColorPickerOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn layout(&mut self, _renderer: &Renderer, viewport: Size) -> layout::Node {
        let inner_h = popup_inner_height();
        let total_h = inner_h + POPUP_PADDING * 2.0;
        let total_w = POPUP_WIDTH;

        // Position below trigger, or above if no room
        let mut x = self.trigger_bounds.x;
        let mut y = self.trigger_bounds.y + self.trigger_bounds.height + 4.0;

        if y + total_h > viewport.height {
            y = self.trigger_bounds.y - total_h - 4.0;
        }
        if x + total_w > viewport.width {
            x = viewport.width - total_w;
        }
        if x < 0.0 {
            x = 0.0;
        }
        if y < 0.0 {
            y = 0.0;
        }

        layout::Node::new(Size::new(total_w, total_h)).move_to(Point::new(x, y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        let inner_origin = Point::new(bounds.x + POPUP_PADDING, bounds.y + POPUP_PADDING);
        let regions = popup_regions(inner_origin);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if !bounds.contains(pos) && !self.trigger_bounds.contains(pos) {
                        // Click outside: dismiss and publish final color
                        self.state.open = false;
                        if let Some(on_change) = self.on_change {
                            shell.publish(on_change(self.state.hsva.to_color()));
                        }
                        shell.capture_event();
                        return;
                    }

                    // Check code click
                    if regions.code.contains(pos) {
                        self.state.color_format = self.state.color_format.next();
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }

                    // Check copy button click
                    if regions.copy.contains(pos) {
                        let text = self.state.color_format.format(self.state.hsva);
                        clipboard.write(iced::advanced::clipboard::Kind::Standard, text);
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }

                    // Check drag targets
                    let target = if regions.sv.contains(pos) {
                        Some(DragTarget::SaturationValue)
                    } else if regions.hue.contains(pos) {
                        Some(DragTarget::Hue)
                    } else {
                        None
                    };

                    if let Some(target) = target {
                        self.state.dragging = Some(target);
                        apply_drag(&mut self.state.hsva, pos, target, &regions);
                        shell.request_redraw();
                        shell.capture_event();
                    }
                } else {
                    // No cursor position — dismiss
                    self.state.open = false;
                    if let Some(on_change) = self.on_change {
                        shell.publish(on_change(self.state.hsva.to_color()));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(target) = self.state.dragging
                    && let Some(pos) = cursor.position()
                {
                    apply_drag(&mut self.state.hsva, pos, target, &regions);
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if self.state.dragging.is_some() =>
            {
                self.state.dragging = None;
                // Publish the final color on release
                if let Some(on_change) = self.on_change {
                    shell.publish(on_change(self.state.hsva.to_color()));
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                self.state.open = false;
                // Publish the final color on dismiss
                if let Some(on_change) = self.on_change {
                    shell.publish(on_change(self.state.hsva.to_color()));
                }
                shell.capture_event();
            }
            _ => {}
        }

        // Suppress unused parameter warnings
        let _ = renderer;
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let _ = cursor;
        let status = Status::Open;
        let style = theme.style(self.class, status);
        let bounds = layout.bounds();

        // Popup background
        if let Some(bg) = style.popup_background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.popup_border,
                    ..renderer::Quad::default()
                },
                bg,
            );
        }

        let inner_origin = Point::new(bounds.x + POPUP_PADDING, bounds.y + POPUP_PADDING);
        let regions = popup_regions(inner_origin);
        let hsva = self.state.hsva;

        // -- SV area (3 overlapping quads) --
        // 1. Base: solid hue color
        renderer.fill_quad(
            renderer::Quad {
                bounds: regions.sv,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            Background::Color(Hsva::hue_color(hsva.h)),
        );
        // 2. Saturation overlay: white (left) → transparent (right)
        let sat_gradient = iced::Gradient::Linear(
            iced::gradient::Linear::new(FRAC_PI_2)
                .add_stop(0.0, Color::WHITE)
                .add_stop(1.0, Color::TRANSPARENT),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: regions.sv,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            Background::Gradient(sat_gradient),
        );
        // 3. Value overlay: transparent (top) → black (bottom)
        let val_gradient = iced::Gradient::Linear(
            iced::gradient::Linear::new(std::f32::consts::PI)
                .add_stop(0.0, Color::TRANSPARENT)
                .add_stop(1.0, Color::BLACK),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: regions.sv,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            Background::Gradient(val_gradient),
        );

        // SV border
        renderer.fill_quad(
            renderer::Quad {
                bounds: regions.sv,
                border: style.bar_border,
                ..renderer::Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );

        // SV crosshair
        let cx = regions.sv.x + hsva.s * regions.sv.width;
        let cy = regions.sv.y + (1.0 - hsva.v) * regions.sv.height;
        draw_crosshair(renderer, cx, cy, &style);

        // -- Hue bar --
        draw_hue_bar(renderer, regions.hue, &style);
        let hx = regions.hue.x + (hsva.h / 360.0) * regions.hue.width;
        draw_bar_indicator(renderer, hx, regions.hue, &style);

        // -- Color code text --
        let code_text = self.state.color_format.format(hsva);
        let text_size = iced::Pixels(theme.text_size() * 0.8125);
        renderer.fill_text(
            iced::advanced::Text {
                content: code_text,
                bounds: regions.code.size(),
                size: text_size,
                line_height: text::LineHeight::Relative(1.2),
                font: Renderer::default_font(renderer),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
            },
            Point::new(regions.code.center_x(), regions.code.center_y()),
            style.text_color,
            regions.code,
        );

        // -- Copy button --
        draw_copy_button(renderer, regions.copy, &style);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.state.dragging.is_some() {
            return mouse::Interaction::Pointer;
        }
        let bounds = layout.bounds();
        if let Some(pos) = cursor.position_over(bounds) {
            let inner_origin = Point::new(bounds.x + POPUP_PADDING, bounds.y + POPUP_PADDING);
            let regions = popup_regions(inner_origin);
            if regions.sv.contains(pos)
                || regions.hue.contains(pos)
                || regions.code.contains(pos)
                || regions.copy.contains(pos)
            {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

// -- Drag logic --

fn apply_drag(hsva: &mut Hsva, pos: Point, target: DragTarget, regions: &PopupRegions) {
    match target {
        DragTarget::SaturationValue => {
            hsva.s = ((pos.x - regions.sv.x) / regions.sv.width).clamp(0.0, 1.0);
            hsva.v = 1.0 - ((pos.y - regions.sv.y) / regions.sv.height).clamp(0.0, 1.0);
        }
        DragTarget::Hue => {
            hsva.h = ((pos.x - regions.hue.x) / regions.hue.width).clamp(0.0, 1.0) * 360.0;
        }
    }
}

// -- Drawing helpers --

fn draw_hue_bar<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    style: &Style,
) {
    let hue_stops: [(f32, Color); 7] = [
        (0.0, Hsva::hue_color(0.0)),
        (1.0 / 6.0, Hsva::hue_color(60.0)),
        (2.0 / 6.0, Hsva::hue_color(120.0)),
        (3.0 / 6.0, Hsva::hue_color(180.0)),
        (4.0 / 6.0, Hsva::hue_color(240.0)),
        (5.0 / 6.0, Hsva::hue_color(300.0)),
        (1.0, Hsva::hue_color(360.0)),
    ];

    for i in 0..6 {
        let x0 = bounds.x + hue_stops[i].0 * bounds.width;
        let x1 = bounds.x + hue_stops[i + 1].0 * bounds.width;
        let seg_bounds =
            Rectangle::new(Point::new(x0, bounds.y), Size::new(x1 - x0, bounds.height));
        let gradient = iced::Gradient::Linear(
            iced::gradient::Linear::new(FRAC_PI_2)
                .add_stop(0.0, hue_stops[i].1)
                .add_stop(1.0, hue_stops[i + 1].1),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: seg_bounds,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            Background::Gradient(gradient),
        );
    }

    // Border
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: style.bar_border,
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );
}

fn draw_crosshair<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    cx: f32,
    cy: f32,
    style: &Style,
) {
    let size = 5.0_f32;
    let outer_bounds = Rectangle::new(
        Point::new(cx - size, cy - size),
        Size::new(size * 2.0, size * 2.0),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: outer_bounds,
            border: Border {
                color: style.handle_border,
                width: 2.0,
                radius: size.into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );
    let inner_size = size - 2.0;
    let inner_bounds = Rectangle::new(
        Point::new(cx - inner_size, cy - inner_size),
        Size::new(inner_size * 2.0, inner_size * 2.0),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: inner_bounds,
            border: Border {
                color: style.handle_color,
                width: 1.5,
                radius: inner_size.into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );
}

fn draw_bar_indicator<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    x: f32,
    bar_bounds: Rectangle,
    style: &Style,
) {
    let w = 4.0_f32;
    let indicator = Rectangle::new(
        Point::new(x - w / 2.0, bar_bounds.y),
        Size::new(w, bar_bounds.height),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: indicator,
            border: Border {
                color: style.handle_border,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(style.handle_color),
    );
}

/// Draws a small "copy" button: an outlined rectangle containing two
/// stacked rounded rectangles that suggest two pieces of paper.
fn draw_copy_button<Renderer: renderer::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    style: &Style,
) {
    // Button outline.
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: style.bar_border,
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );

    // Glyph: two overlapping squares.
    let glyph_size = 9.0_f32;
    let offset = 3.0_f32;
    let total_w = glyph_size + offset;
    let total_h = glyph_size + offset;

    let origin_x = bounds.center_x() - total_w / 2.0;
    let origin_y = bounds.center_y() - total_h / 2.0;

    // Back square (upper-right).
    let back = Rectangle::new(
        Point::new(origin_x + offset, origin_y),
        Size::new(glyph_size, glyph_size),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: back,
            border: Border {
                color: style.text_color,
                width: 1.0,
                radius: 1.5.into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );

    // Front square (lower-left), filled with the popup background so it
    // visually overlaps the back square.
    let front = Rectangle::new(
        Point::new(origin_x, origin_y + offset),
        Size::new(glyph_size, glyph_size),
    );
    let front_fill = match style.popup_background {
        Some(Background::Color(c)) => c,
        _ => Color::TRANSPARENT,
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: front,
            border: Border {
                color: style.text_color,
                width: 1.0,
                radius: 1.5.into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(front_fill),
    );
}

// -- Into<Element> --

impl<'a, Message, Theme, Renderer> From<ColorPicker<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + 'a,
{
    fn from(picker: ColorPicker<'a, Message, Theme, Renderer>) -> Self {
        Self::new(picker)
    }
}
