//! A horizontal progress indicator with determinate and indeterminate
//! modes.
//!
//! [`Progress`] fills the available width with a rounded track. In
//! **determinate** mode it fills a fraction of the track proportional
//! to a value in `0.0..=1.0`. In **indeterminate** mode a shorter
//! segment loops continuously across the track in one direction,
//! conveying ongoing work of unknown duration; this mode animates by
//! requesting redraws from the runtime.
//!
//! The bar color is driven by a semantic [`Color`] token that maps
//! onto the active theme's color groups, defaulting to
//! [`Color::Info`].
//!
//! # Docking and elevation
//!
//! A bar may be [docked](Dock) flush against the top or bottom edge of
//! its layout slot. Docked bars flatten the corners that meet the edge
//! and, by default, gain a drop shadow (elevation) so they appear to
//! float above the content they cap. Elevation can be forced on or off
//! independently of docking via [`Progress::elevated`].
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::progress::{Progress, Color, Dock};
//!
//! // Determinate: 60% complete.
//! let p = Progress::determinate(0.6);
//!
//! // Indeterminate, success-tinted, docked to the top with a shadow.
//! let p = Progress::indeterminate().color(Color::Success).dock(Dock::Top);
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
use iced::{Element, Event, Length, Rectangle, Size, Vector, window};

use crate::{Space, SpacingBase};

/// Duration of one full loop of the indeterminate indicator.
const LOOP_PERIOD: f32 = 1.4;

/// Fraction of the track width occupied by the indeterminate segment.
const SEGMENT_FRACTION: f32 = 0.35;

/// Default thickness (height) of the progress track.
const DEFAULT_THICKNESS: Space = Space::sx(0.5);

/// The edge an indicator is docked against.
///
/// Docking flattens the corners flush with the edge and, by default,
/// elevates the bar with a drop shadow. [`None`](Dock::None) leaves the
/// bar inline with default rounded corners and no shadow.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dock {
    /// Inline, not docked: rounded corners, no shadow by default.
    #[default]
    None,
    /// Docked flush against the top edge.
    Top,
    /// Docked flush against the bottom edge.
    Bottom,
}

impl Dock {
    /// Whether this is a docked edge (not [`Dock::None`]).
    fn is_docked(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the shadow offset vector pointing away from the docked
    /// edge (into the content the bar caps). Zero when not docked.
    fn shadow_offset(self) -> Vector {
        match self {
            Self::None => Vector::new(0.0, 0.0),
            Self::Top => Vector::new(0.0, 2.0),
            Self::Bottom => Vector::new(0.0, -2.0),
        }
    }

    /// Zeros the border radius on the corners that sit flush against
    /// the docked edge, leaving the opposite corners rounded.
    fn apply_border_radius(self, radius: &mut iced::border::Radius) {
        match self {
            Self::None => {}
            Self::Top => {
                radius.top_left = 0.0;
                radius.top_right = 0.0;
            }
            Self::Bottom => {
                radius.bottom_left = 0.0;
                radius.bottom_right = 0.0;
            }
        }
    }
}

/// The progress mode: a known fraction, or indeterminate.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    /// A fraction in `0.0..=1.0`.
    Determinate(f32),
    /// Unknown duration; an animated looping segment.
    Indeterminate,
}

/// Internal widget state.
///
/// `start` is the instant the indeterminate animation began; `None`
/// until the first [`window::Event::RedrawRequested`] is observed,
/// which keeps the first rendered frame deterministic (phase 0).
///
/// `spacing` caches the theme's spacing base, refreshed in
/// [`Widget::draw`] each frame so that [`Widget::layout`] can resolve
/// spacing tokens even though the theme is not available there.
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

/// A horizontal progress indicator.
pub struct Progress<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    mode: Mode,
    thickness: Space,
    color: Color,
    dock: Dock,
    elevated: Option<bool>,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Progress<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a determinate progress bar filled to `value`, a
    /// fraction in `0.0..=1.0` (values outside the range are clamped).
    pub fn determinate(value: f32) -> Self {
        Self::with_mode(Mode::Determinate(value.clamp(0.0, 1.0)))
    }

    /// Creates an indeterminate progress bar with a continuously
    /// looping segment.
    pub fn indeterminate() -> Self {
        Self::with_mode(Mode::Indeterminate)
    }

    fn with_mode(mode: Mode) -> Self {
        Self {
            mode,
            thickness: DEFAULT_THICKNESS,
            color: Color::default(),
            dock: Dock::default(),
            elevated: None,
            class: Theme::default(),
        }
    }

    /// Sets the thickness (height) of the track.
    ///
    /// Defaults to [`Space::sx(0.5)`](Space::sx), which resolves to
    /// `4.0` logical pixels at the default theme spacing.
    pub fn thickness(mut self, thickness: impl Into<Space>) -> Self {
        self.thickness = thickness.into();
        self
    }

    /// Sets the semantic [`Color`] token used to tint the bar.
    ///
    /// Defaults to [`Color::Info`].
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Docks the bar flush against the given edge.
    ///
    /// Docking flattens the corners that meet the edge and, by
    /// default, elevates the bar with a drop shadow (see
    /// [`Progress::elevated`] to override). Defaults to
    /// [`Dock::None`].
    pub fn dock(mut self, dock: Dock) -> Self {
        self.dock = dock;
        self
    }

    /// Forces the elevation (drop shadow) on or off, overriding the
    /// dock-driven default.
    ///
    /// When unset, the bar is elevated if and only if it is
    /// [docked](Progress::dock). Pass `true` to always show the
    /// shadow, or `false` to suppress it even when docked.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = Some(elevated);
        self
    }

    /// Resolves whether the bar should be elevated: the explicit
    /// override if set, otherwise elevated iff docked.
    fn is_elevated(&self) -> bool {
        self.elevated.unwrap_or_else(|| self.dock.is_docked())
    }

    /// Sets the style class for this progress bar.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Progress<'a, Theme>
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
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Resolve themed `Space::sx` tokens against the spacing base
        // cached from the most recent `draw`. Seeded with the default
        // base, so the very first frame is reasonable; subsequent
        // frames track the live theme spacing.
        let spacing = tree.state.downcast_ref::<State>().spacing.get();
        let thickness = self.thickness.resolve(spacing);
        let width = limits.max().width;
        layout::Node::new(Size::new(width, thickness))
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
        // Cache the spacing base so `layout` can resolve tokens.
        state.spacing.set(theme.spacing());

        let style = Catalog::style(
            theme,
            &self.class,
            self.color,
            self.dock,
            self.is_elevated(),
        );
        let bounds = layout.bounds();

        // Track corner radius: a pill by default, flattened on the
        // docked edge.
        let mut radius = iced::border::Radius::from(bounds.height / 2.0);
        self.dock.apply_border_radius(&mut radius);

        // Track (with elevation shadow).
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    radius,
                    ..iced::Border::default()
                },
                shadow: style.shadow,
                ..renderer::Quad::default()
            },
            iced::Background::Color(style.track),
        );

        // Compute the filled segment in track-local coordinates.
        let (seg_x, seg_width) = match self.mode {
            Mode::Determinate(value) => (0.0, bounds.width * value),
            Mode::Indeterminate => {
                let phase = match state.start {
                    Some(start) => start.elapsed().as_secs_f32() / LOOP_PERIOD,
                    None => 0.0,
                };
                // Monotonic 0->1 ramp that wraps: one continuous
                // direction, no back-and-forth.
                let t = phase.fract();

                let seg_width = bounds.width * SEGMENT_FRACTION;
                // The segment's leading edge travels from fully off the
                // left (`-seg_width`) to fully off the right
                // (`bounds.width`), so it enters from the left, crosses,
                // exits right, and loops seamlessly. The drawn rect is
                // clipped to the track below.
                let span = bounds.width + seg_width;
                let lead = -seg_width + t * span;
                (lead, seg_width)
            }
        };

        if seg_width <= 0.0 {
            return;
        }

        // Clip the bar to the track bounds (matters for the looping
        // indeterminate segment as it enters/exits the edges).
        let left = (bounds.x + seg_x).max(bounds.x);
        let right = (bounds.x + seg_x + seg_width).min(bounds.x + bounds.width);
        let drawn_width = right - left;

        if drawn_width <= 0.0 {
            return;
        }

        let mut bar_radius = iced::border::Radius::from(bounds.height / 2.0);
        self.dock.apply_border_radius(&mut bar_radius);

        let bar_rect = Rectangle {
            x: left,
            y: bounds.y,
            width: drawn_width,
            height: bounds.height,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: bar_rect,
                border: iced::Border {
                    radius: bar_radius,
                    ..iced::Border::default()
                },
                shadow: iced::Shadow::default(),
                ..renderer::Quad::default()
            },
            iced::Background::Color(style.bar),
        );
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
        // Only the indeterminate mode animates.
        if !matches!(self.mode, Mode::Indeterminate) {
            return;
        }

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = tree.state.downcast_mut::<State>();
            if state.start.is_none() {
                state.start = Some(Instant::now());
            }
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

impl<'a, Message, Theme, Renderer> From<Progress<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(progress: Progress<'a, Theme>) -> Self {
        Element::new(progress)
    }
}
