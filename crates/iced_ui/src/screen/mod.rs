//! A device-screen container that constrains its content to a
//! specific aspect ratio with presets for desktop and mobile devices.
//!
//! The [`Screen`] widget wraps content in a [`Card`] and enforces an
//! aspect ratio during layout, making it ideal for demonstrating how
//! a UI would appear on different devices.
//!
//! ```no_run
//! # use iced::widget::text;
//! # type Message = ();
//! use iced_ui::screen::{Screen, Mode};
//!
//! let desktop: iced_ui::screen::Screen<'_, Message, iced_ui::Theme> =
//!     Screen::new(text("Hello"));
//!
//! let phone: iced_ui::screen::Screen<'_, Message, iced_ui::Theme> =
//!     Screen::new(text("Hello")).mode(Mode::MobilePortrait);
//! ```

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::svg as advanced_svg;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Padding, Rectangle, Size};

use crate::card::Card;
use crate::theme::SpacingBase;

/// Device mode presets controlling the default aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// 16:9 landscape (default).
    #[default]
    Desktop,
    /// 20:9 landscape.
    MobileLandscape,
    /// 9:20 portrait.
    MobilePortrait,
}

impl Mode {
    /// Default aspect ratio `(width, height)` for this mode.
    const fn default_aspect(self) -> (u32, u32) {
        match self {
            Self::Desktop => (16, 9),
            Self::MobileLandscape => (20, 9),
            Self::MobilePortrait => (9, 20),
        }
    }

    /// Default max width for this mode (`None` means no limit).
    const fn default_max_width(self) -> Option<f32> {
        match self {
            Self::Desktop => None,
            Self::MobileLandscape => None,
            Self::MobilePortrait => Some(360.0),
        }
    }
}

/// A device-screen container that constrains its content to an aspect
/// ratio. Wraps content in a [`Card`] with zero padding.
///
/// # Defaults
///
/// - Mode: [`Mode::Desktop`]
/// - Aspect ratio: 16:9
/// - Max width: unbounded (except [`Mode::MobilePortrait`] which
///   defaults to 360px)
#[must_use]
pub struct Screen<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: crate::card::Catalog + SpacingBase,
    Renderer: iced::advanced::Renderer
        + iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>
        + advanced_svg::Renderer,
{
    card: Card<'a, Message, Theme, Renderer>,
    aspect: (u32, u32),
    max_width: f32,
}

impl<'a, Message, Theme, Renderer> Screen<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: crate::card::Catalog + SpacingBase + 'a,
    Renderer: iced::advanced::Renderer
        + iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>
        + advanced_svg::Renderer
        + 'a,
{
    /// Create a new [`Screen`] with the given content in [`Mode::Desktop`].
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let mode = Mode::Desktop;
        let card = Card::new(content).padding(Padding::ZERO);
        Self {
            card,
            aspect: mode.default_aspect(),
            max_width: mode.default_max_width().unwrap_or(f32::INFINITY),
        }
    }

    /// Set the device mode, updating the aspect ratio and max width
    /// to that mode's defaults.
    pub fn mode(mut self, mode: Mode) -> Self {
        self.aspect = mode.default_aspect();
        self.max_width = mode.default_max_width().unwrap_or(f32::INFINITY);
        self
    }

    /// Override the aspect ratio as `(width, height)`.
    pub fn aspect(mut self, width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "aspect ratio must be non-zero");
        self.aspect = (width, height);
        self
    }

    /// Set the maximum width in logical pixels.
    pub fn max_width(mut self, max_width: impl Into<f32>) -> Self {
        self.max_width = max_width.into();
        self
    }

    /// Compute the aspect-constrained size given available bounds.
    fn compute_size(&self, max: Size) -> Size {
        let (aw, ah) = (self.aspect.0 as f32, self.aspect.1 as f32);
        let ratio = aw / ah;

        // Try fitting by width first
        let w = max.width.min(self.max_width);
        let h = w / ratio;

        if h <= max.height {
            Size::new(w, h)
        } else {
            // Fit by height instead
            let h = max.height;
            let w = (h * ratio).min(self.max_width);
            Size::new(w, h)
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Screen<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: crate::card::Catalog + SpacingBase + 'a,
    Renderer: iced::advanced::Renderer
        + iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>
        + advanced_svg::Renderer
        + 'a,
{
    fn tag(&self) -> tree::Tag {
        self.card.tag()
    }

    fn state(&self) -> tree::State {
        self.card.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.card.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.card.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let size = self.compute_size(max);

        // Layout the inner card with exact size limits
        let card_limits = layout::Limits::new(Size::ZERO, size);
        let mut card_node = self.card.layout(tree, renderer, &card_limits);
        card_node = card_node.move_to(iced::Point::ORIGIN);

        layout::Node::with_children(size, vec![card_node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let card_layout = layout.children().next().unwrap();
        self.card
            .draw(tree, renderer, theme, style, card_layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let card_layout = layout.children().next().unwrap();
        self.card.operate(tree, card_layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let card_layout = layout.children().next().unwrap();
        self.card.update(
            tree,
            event,
            card_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let card_layout = layout.children().next().unwrap();
        self.card
            .mouse_interaction(tree, card_layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let card_layout = layout.children().next().unwrap();
        self.card
            .overlay(tree, card_layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Screen<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: crate::card::Catalog + SpacingBase + 'a,
    Renderer: iced::advanced::Renderer
        + iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>
        + advanced_svg::Renderer
        + 'a,
{
    fn from(screen: Screen<'a, Message, Theme, Renderer>) -> Self {
        Self::new(screen)
    }
}
