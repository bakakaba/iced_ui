//! The visible surface of a [`Tooltip`](super::Tooltip).
//!
//! [`Bubble`] is a private helper widget passed to
//! [`iced::widget::Tooltip`] as its *content*, while iced's own box is
//! rendered invisibly. It draws the themed rounded-rect background and
//! shadow, an optional directional caret pointing at the trigger, and
//! the user's tooltip content.
//!
//! The caret is a triangular SVG tinted to the bubble background, with
//! one pre-oriented shape per [`Position`], so all four directions are
//! rendered from the same crisp, font-independent triangle. Drawing the
//! caret requires the `svg` feature (enabled by default); without it
//! the caret is omitted, but the edge spacing it would occupy is still
//! reserved so the bubble's geometry is identical either way.
//!
//! iced draws tooltip content with an infinite viewport, so the caret
//! may protrude past the bubble's background rectangle toward the
//! trigger without being clipped.

use std::cell::Cell;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::{Element, Event, Length, Point, Rectangle, Size, mouse};

use super::Position;
use super::style::{Catalog, Style};
use crate::{Space, SpacingBase};

/// Span of the caret along the bubble edge, as a spacing token.
const CARET_SPAN: Space = Space::sx(1.75);

/// Depth the caret protrudes from the bubble edge, as a spacing token.
const CARET_DEPTH: Space = Space::sx(0.875);

/// Renderer capability required to draw the caret.
///
/// The caret is an SVG, so it needs [`iced::advanced::svg::Renderer`]
/// — but only when the `svg` feature is enabled. Gating the bound here
/// keeps the [`Bubble`] widget usable (sans caret) on renderers built
/// without SVG support.
#[cfg(feature = "svg")]
pub(super) trait CaretRenderer: iced::advanced::svg::Renderer {}
#[cfg(feature = "svg")]
impl<T: iced::advanced::svg::Renderer> CaretRenderer for T {}

/// Renderer capability required to draw the caret (no-op without the
/// `svg` feature).
#[cfg(not(feature = "svg"))]
pub(super) trait CaretRenderer {}
#[cfg(not(feature = "svg"))]
impl<T> CaretRenderer for T {}

/// Handle for the caret triangle SVG pointing in the given direction.
///
/// Four pre-oriented shapes are used (rather than one shape rotated at
/// draw time) because iced's SVG renderer does not reliably rotate a
/// small rasterized vector; authoring each orientation directly keeps
/// every caret crisp and identical in shape. The `up`/`down` shapes use
/// a wide 2:1 viewBox; `left`/`right` a tall 1:2 viewBox, matching the
/// caret's `span x depth` footprint per edge. The fill is overridden
/// per-draw via [`Svg::color`] to match the bubble.
#[cfg(feature = "svg")]
fn caret_handle(position: Position) -> iced::advanced::svg::Handle {
    use std::sync::OnceLock;

    // Triangles fill their viewBox: base along the bubble edge, apex
    // pointing toward the trigger.
    const UP: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10"><polygon points="10,0 20,10 0,10"/></svg>"#;
    const DOWN: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10"><polygon points="0,0 20,0 10,10"/></svg>"#;
    const LEFT: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 20"><polygon points="0,10 10,0 10,20"/></svg>"#;
    const RIGHT: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 20"><polygon points="10,10 0,0 0,20"/></svg>"#;

    static UP_H: OnceLock<iced::advanced::svg::Handle> = OnceLock::new();
    static DOWN_H: OnceLock<iced::advanced::svg::Handle> = OnceLock::new();
    static LEFT_H: OnceLock<iced::advanced::svg::Handle> = OnceLock::new();
    static RIGHT_H: OnceLock<iced::advanced::svg::Handle> = OnceLock::new();

    let (cell, bytes): (&OnceLock<_>, &[u8]) = match position {
        // Bubble below trigger -> caret on top edge, points up.
        Position::Bottom => (&UP_H, UP),
        // Bubble above trigger -> caret on bottom edge, points down.
        Position::Top => (&DOWN_H, DOWN),
        // Bubble right of trigger -> caret on left edge, points left.
        Position::Right => (&LEFT_H, LEFT),
        // Bubble left of trigger -> caret on right edge, points right.
        Position::Left => (&RIGHT_H, RIGHT),
        Position::FollowCursor => (&UP_H, UP),
    };
    cell.get_or_init(|| iced::advanced::svg::Handle::from_memory(bytes.to_vec()))
        .clone()
}

/// Internal cache of theme tokens resolved in [`Widget::draw`] and read
/// back in [`Widget::layout`] (which has no `&Theme`).
#[derive(Debug)]
struct State {
    /// Inner padding around the content, cached from the theme spacing.
    padding: Cell<f32>,
    /// Caret span (along the edge), cached from the theme spacing.
    caret_span: Cell<f32>,
    /// Caret depth (protrusion), cached from the theme spacing.
    caret_depth: Cell<f32>,
}

impl Default for State {
    fn default() -> Self {
        // Seeded so the first frame (before any draw) is reasonable;
        // refreshed against the live theme in `draw`.
        let base = crate::Theme::DEFAULT_SPACING;
        Self {
            padding: Cell::new(Space::sx(1.0).resolve(base)),
            caret_span: Cell::new(CARET_SPAN.resolve(base)),
            caret_depth: Cell::new(CARET_DEPTH.resolve(base)),
        }
    }
}

/// The visible tooltip surface: themed background + caret + content.
#[allow(missing_debug_implementations)]
pub(super) struct Bubble<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,
    position: Position,
    caret: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Bubble<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    /// Wraps `content` as a themed tooltip bubble anchored per
    /// `position`, optionally drawing a caret.
    pub(super) fn new(
        content: Element<'a, Message, Theme, Renderer>,
        position: Position,
        caret: bool,
        class: Theme::Class<'a>,
    ) -> Self {
        Self {
            content,
            position,
            caret,
            class,
        }
    }

    /// Whether a caret should be drawn for this bubble.
    ///
    /// `FollowCursor` has no fixed trigger edge to point at, so it never
    /// draws a caret. Note this gates only *drawing*: the caret's edge
    /// spacing is always reserved in [`Widget::layout`] so the bubble's
    /// geometry does not change when the caret is hidden.
    fn draws_caret(&self) -> bool {
        self.caret && self.position != Position::FollowCursor
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Bubble<'a, Message, Theme, Renderer>
where
    Theme: Catalog + SpacingBase,
    Renderer: renderer::Renderer + text::Renderer + CaretRenderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
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
        let state = tree.state.downcast_ref::<State>();
        let padding = state.padding.get();
        // Always reserve the caret's protrusion depth on the trigger
        // side — independent of whether the caret is actually drawn — so
        // the bubble geometry is stable across feature/`caret` configs.
        let caret = state.caret_depth.get();

        // Vertical caret (Top/Bottom) extends the height; horizontal
        // caret (Left/Right) extends the width.
        let (caret_w, caret_h) = match self.position {
            Position::Left | Position::Right => (caret, 0.0),
            _ => (0.0, caret),
        };

        let content_limits =
            limits.shrink(Size::new(padding * 2.0 + caret_w, padding * 2.0 + caret_h));
        let content =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &content_limits);
        let content_size = content.size();

        let width = content_size.width + padding * 2.0 + caret_w;
        let height = content_size.height + padding * 2.0 + caret_h;

        // Offset the content past the caret when the caret sits on the
        // leading (top/left) edge.
        let offset_x = padding
            + if self.position == Position::Right {
                caret
            } else {
                0.0
            };
        let offset_y = padding
            + if self.position == Position::Bottom {
                caret
            } else {
                0.0
            };

        let content = content.move_to(Point::new(offset_x, offset_y));

        layout::Node::with_children(Size::new(width, height), vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let style: Style = Catalog::style(theme, &self.class);

        // Cache theme tokens for the next `layout` pass.
        let state = tree.state.downcast_ref::<State>();
        let base = SpacingBase::spacing(theme);
        let padding = Space::sx(1.0).resolve(base);
        let caret_span = CARET_SPAN.resolve(base);
        let caret_depth = CARET_DEPTH.resolve(base);
        state.padding.set(padding);
        state.caret_span.set(caret_span);
        state.caret_depth.set(caret_depth);

        let bounds = layout.bounds();
        // Layout always reserves the caret depth on the trigger side.
        let caret = caret_depth;

        // The background rectangle excludes the reserved caret strip so
        // the caret appears to protrude from its edge.
        let bg_bounds = match self.position {
            Position::Top => Rectangle {
                height: bounds.height - caret,
                ..bounds
            },
            Position::Bottom => Rectangle {
                y: bounds.y + caret,
                height: bounds.height - caret,
                ..bounds
            },
            Position::Left => Rectangle {
                width: bounds.width - caret,
                ..bounds
            },
            Position::Right => Rectangle {
                x: bounds.x + caret,
                width: bounds.width - caret,
                ..bounds
            },
            Position::FollowCursor => bounds,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: bg_bounds,
                border: style.border,
                shadow: style.shadow,
                ..renderer::Quad::default()
            },
            style.background,
        );

        if self.draws_caret() {
            self.draw_caret(renderer, &style, bg_bounds, caret_span, caret_depth);
        }

        let defaults = renderer::Style {
            text_color: style.text_color,
        };
        let _ = renderer_style;

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &defaults,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }
}

impl<'a, Message, Theme, Renderer> Bubble<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: CaretRenderer,
{
    /// Draws the caret SVG, tinted to the bubble background and rotated
    /// to point from `bg_bounds` toward the trigger.
    ///
    /// A no-op without the `svg` feature; the caret's edge spacing is
    /// still reserved in [`Widget::layout`] regardless.
    #[cfg(feature = "svg")]
    fn draw_caret(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        bg_bounds: Rectangle,
        span: f32,
        depth: f32,
    ) {
        use iced::advanced::svg::Svg;

        // Each direction has its own pre-oriented triangle SVG (see
        // `caret_handle`), so no rotation is needed — the SVG is simply
        // scaled into a rect sitting flush against the trigger-facing
        // edge with its apex protruding by `depth`. Up/Down carets are
        // `span` wide by `depth` tall; Left/Right swap the axes.
        let rect = match self.position {
            // Caret on the bubble's top edge, pointing up.
            Position::Bottom => Rectangle {
                x: bg_bounds.center_x() - span / 2.0,
                y: bg_bounds.y - depth,
                width: span,
                height: depth,
            },
            // Caret on the bubble's bottom edge, pointing down.
            Position::Top => Rectangle {
                x: bg_bounds.center_x() - span / 2.0,
                y: bg_bounds.y + bg_bounds.height,
                width: span,
                height: depth,
            },
            // Caret on the bubble's left edge, pointing left.
            Position::Right => Rectangle {
                x: bg_bounds.x - depth,
                y: bg_bounds.center_y() - span / 2.0,
                width: depth,
                height: span,
            },
            // Caret on the bubble's right edge, pointing right.
            Position::Left => Rectangle {
                x: bg_bounds.x + bg_bounds.width,
                y: bg_bounds.center_y() - span / 2.0,
                width: depth,
                height: span,
            },
            Position::FollowCursor => return,
        };

        renderer.draw_svg(
            Svg {
                handle: caret_handle(self.position),
                color: Some(style.background),
                rotation: iced::Radians(0.0),
                opacity: 1.0,
            },
            rect,
            rect,
        );
    }

    /// Draws the caret (no-op without the `svg` feature).
    #[cfg(not(feature = "svg"))]
    fn draw_caret(
        &self,
        _renderer: &mut Renderer,
        _style: &Style,
        _bg_bounds: Rectangle,
        _span: f32,
        _depth: f32,
    ) {
    }
}

impl<'a, Message, Theme, Renderer> From<Bubble<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer + text::Renderer + CaretRenderer + 'a,
{
    fn from(bubble: Bubble<'a, Message, Theme, Renderer>) -> Self {
        Element::new(bubble)
    }
}
