//! An indeterminate activity indicator.
//!
//! [`Spinner`] renders a ring of small dots whose opacity "chases"
//! around the circle, conveying that work is in progress without a
//! known completion percentage. It animates continuously by requesting
//! redraws from the runtime.
//!
//! The element color is driven by a semantic [`Color`] token that maps
//! onto the active theme's color groups, defaulting to
//! [`Color::Info`].
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::spinner::{Spinner, Color};
//!
//! // Default info-colored spinner.
//! let s = Spinner::new();
//!
//! // A success-colored, larger spinner.
//! let s = Spinner::new().color(Color::Success).size(iced_ui::Space::sx(4.0));
//! ```

mod style;

pub use style::{Catalog, Color, Style, StyleFn, default};

use std::cell::Cell;
use std::time::Instant;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Point, Rectangle, Size, window};

use crate::{Space, SpacingBase};

/// Number of dots arranged around the ring.
const DOT_COUNT: usize = 8;

/// Duration of one full rotation.
const PERIOD: f32 = 1.0;

/// Default diameter of the spinner.
const DEFAULT_SIZE: Space = Space::sx(3.0);

/// Internal widget state.
///
/// `start` is the instant the animation began; `None` until the first
/// [`window::Event::RedrawRequested`] is observed, which keeps the very
/// first rendered frame deterministic (phase 0).
///
/// `spacing` caches the theme's spacing base, refreshed in
/// [`Widget::draw`] each frame so that [`Widget::layout`] can resolve
/// the diameter token even though the theme is not available there.
#[derive(Debug)]
struct State {
    start: Option<Instant>,
    spacing: Cell<u8>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            start: None,
            spacing: Cell::new(crate::Theme::DEFAULT_SPACING),
        }
    }
}

/// An indeterminate, continuously animating activity indicator.
pub struct Spinner<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    size: Space,
    color: Color,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Spinner<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a new spinner with default size and the
    /// [`Color::Info`] color token.
    pub fn new() -> Self {
        Self {
            size: DEFAULT_SIZE,
            color: Color::default(),
            class: Theme::default(),
        }
    }

    /// Sets the overall diameter of the spinner.
    ///
    /// Defaults to [`Space::sx(3.0)`](Space::sx), which resolves to
    /// `24.0` logical pixels at the default theme spacing.
    pub fn size(mut self, size: impl Into<Space>) -> Self {
        self.size = size.into();
        self
    }

    /// Sets the semantic [`Color`] token used to tint the spinner.
    ///
    /// Defaults to [`Color::Info`].
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the style class for this spinner.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Theme> Default for Spinner<'a, Theme>
where
    Theme: Catalog,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Spinner<'a, Theme>
where
    Theme: Catalog + SpacingBase,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Resolve the themed `Space::sx` diameter against the spacing
        // base cached from the most recent `draw`. Seeded with the
        // default base, so the first frame is reasonable; subsequent
        // frames track the live theme spacing.
        let spacing = tree.state.downcast_ref::<State>().spacing.get();
        let diameter = self.size.resolve(spacing);
        let size = Size::new(diameter, diameter);
        layout::Node::new(limits.resolve(Length::Shrink, Length::Shrink, size))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let style = Catalog::style(theme, &self.class, self.color);
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        // Cache the spacing base so `layout` can resolve the diameter
        // token against the live theme on subsequent frames.
        state.spacing.set(theme.spacing());

        // Resolve phase in [0, 1). Deterministic 0 before the first
        // redraw is observed.
        let phase = match state.start {
            Some(start) => {
                let elapsed = start.elapsed().as_secs_f32();
                (elapsed / PERIOD).fract()
            }
            None => 0.0,
        };

        let diameter = bounds.width.min(bounds.height);
        let center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );
        let ring_radius = diameter / 2.0;
        // Dot radius scales with the spinner so it stays proportional.
        let dot_radius = (diameter * 0.12).max(1.0);
        let orbit = (ring_radius - dot_radius).max(0.0);

        renderer.with_layer(*viewport, |renderer| {
            for i in 0..DOT_COUNT {
                let frac = i as f32 / DOT_COUNT as f32;
                let angle = frac * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;

                let dx = angle.cos() * orbit;
                let dy = angle.sin() * orbit;

                // Opacity trails the leading dot: the dot at `phase`
                // is brightest, fading around the ring.
                let mut delta = frac - phase;
                if delta < 0.0 {
                    delta += 1.0;
                }
                let opacity = 0.15 + 0.85 * (1.0 - delta);

                let dot_rect = Rectangle {
                    x: center.x + dx - dot_radius,
                    y: center.y + dy - dot_radius,
                    width: dot_radius * 2.0,
                    height: dot_radius * 2.0,
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: dot_rect,
                        border: iced::Border {
                            radius: dot_radius.into(),
                            ..iced::Border::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..renderer::Quad::default()
                    },
                    iced::Background::Color(iced::Color {
                        a: style.color.a * opacity,
                        ..style.color
                    }),
                );
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = tree.state.downcast_mut::<State>();
            if state.start.is_none() {
                state.start = Some(Instant::now());
            }
            // Keep animating.
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, Message, Theme, Renderer> From<Spinner<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(spinner: Spinner<'a, Theme>) -> Self {
        Element::new(spinner)
    }
}
