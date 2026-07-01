//! A floating [`Tooltip`] that reveals contextual content on hover.
//!
//! [`Tooltip`] builds on [`iced::widget::Tooltip`], reusing iced's
//! proven hover detection, delay, and overlay positioning. iced's own
//! bubble box is rendered invisibly; the visible surface — a themed
//! rounded rectangle with a shadow and a directional caret glyph
//! pointing at the trigger — is drawn by a private `Bubble` widget
//! passed as the tooltip content. The trigger and the bubble content
//! are both arbitrary [`Element`]s, so a tooltip can reveal anything
//! from a short label to a rich card.
//!
//! The caret is enabled by default; disable it with
//! [`Tooltip::caret`]. [`Position::FollowCursor`] never draws a caret,
//! as it has no fixed trigger edge to point at.
//!
//! # Example
//!
//! ```no_run
//! use iced::widget::text;
//! use iced_ui::Tooltip;
//! use iced_ui::tooltip::Position;
//!
//! # #[derive(Debug, Clone)]
//! # enum Message {}
//! let tooltip: iced::Element<'_, Message, iced_ui::Theme> =
//!     Tooltip::new(text("Hover me"), text("Extra detail"), Position::Top).into();
//! ```

mod bubble;
mod style;

pub use style::{Catalog, Style, StyleFn, default};

use std::time::Duration;

use iced::widget::{container, tooltip as iced_tooltip};
use iced::{Element, Pixels};

/// The placement of a [`Tooltip`] bubble relative to its trigger.
///
/// Mirrors [`iced::widget::tooltip::Position`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// Above the trigger.
    #[default]
    Top,
    /// Below the trigger.
    Bottom,
    /// To the left of the trigger.
    Left,
    /// To the right of the trigger.
    Right,
    /// Anchored to the cursor, tracking it while hovering.
    FollowCursor,
}

impl Position {
    /// Maps to the equivalent [`iced::widget::tooltip::Position`].
    fn to_iced(self) -> iced_tooltip::Position {
        match self {
            Self::Top => iced_tooltip::Position::Top,
            Self::Bottom => iced_tooltip::Position::Bottom,
            Self::Left => iced_tooltip::Position::Left,
            Self::Right => iced_tooltip::Position::Right,
            Self::FollowCursor => iced_tooltip::Position::FollowCursor,
        }
    }
}

/// A widget that overlays a floating bubble when its trigger is hovered.
///
/// Build one with [`Tooltip::new`], then convert it into an
/// [`Element`] (directly or via a layout container). The bubble's
/// appearance is driven by the active [`Theme`](crate::Theme) through a
/// [`Catalog`]; override it per-instance with [`Tooltip::style`] or
/// [`Tooltip::class`].
#[allow(missing_debug_implementations)]
pub struct Tooltip<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
{
    /// The always-visible trigger element.
    content: Element<'a, Message, Theme, Renderer>,
    /// The bubble shown while the trigger is hovered.
    tooltip: Element<'a, Message, Theme, Renderer>,
    position: Position,
    gap: Option<f32>,
    padding: Option<f32>,
    delay: Duration,
    snap_within_viewport: bool,
    caret: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Tooltip<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    /// Creates a new [`Tooltip`] revealing `tooltip` when `content` is
    /// hovered, placed at `position`.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
        position: Position,
    ) -> Self {
        Self {
            content: content.into(),
            tooltip: tooltip.into(),
            position,
            gap: None,
            padding: None,
            delay: Duration::ZERO,
            snap_within_viewport: true,
            caret: true,
            class: Theme::default(),
        }
    }

    /// Sets the gap between the trigger and its bubble.
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = Some(gap.into().0);
        self
    }

    /// Sets the inner padding of the bubble.
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = Some(padding.into().0);
        self
    }

    /// Sets the delay before the bubble appears once hovering begins.
    ///
    /// Defaults to [`Duration::ZERO`] (shown immediately).
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets whether the bubble is kept within the viewport bounds.
    ///
    /// Enabled by default.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }

    /// Sets whether a directional caret is drawn pointing at the
    /// trigger.
    ///
    /// Enabled by default. Has no effect for
    /// [`Position::FollowCursor`], which has no fixed edge to point at.
    ///
    /// The caret is rendered as an SVG and therefore requires the `svg`
    /// feature (enabled by default); without it no caret is drawn.
    /// Either way the caret's edge spacing is reserved, so toggling this
    /// (or the feature) does not change the bubble's size or position.
    pub fn caret(mut self, caret: bool) -> Self {
        self.caret = caret;
        self
    }

    /// Sets the style of the [`Tooltip`] bubble.
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Tooltip`] bubble.
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> From<Tooltip<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + container::Catalog + crate::SpacingBase + 'a,
    <Theme as container::Catalog>::Class<'a>: From<container::StyleFn<'a, Theme>>,
    Renderer:
        iced::advanced::Renderer + iced::advanced::text::Renderer + bubble::CaretRenderer + 'a,
{
    fn from(tooltip: Tooltip<'a, Message, Theme, Renderer>) -> Self {
        let Tooltip {
            content,
            tooltip: bubble,
            position,
            gap,
            padding,
            delay,
            snap_within_viewport,
            caret,
            class,
        } = tooltip;

        // The visible tooltip surface is drawn by `Bubble` (background,
        // shadow, caret, content), which owns the style `class` and
        // resolves it from the live `&Theme` in its `draw`. iced's own
        // tooltip box is rendered fully invisible so only our bubble
        // shows, while iced retains all hover / delay / positioning /
        // snapping behavior.
        let surface = bubble::Bubble::new(bubble, position, caret, class);

        let mut inner = iced_tooltip::Tooltip::new(content, surface, position.to_iced())
            .delay(delay)
            .snap_within_viewport(snap_within_viewport)
            .style(|_theme: &Theme| container::Style {
                text_color: None,
                background: None,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            });

        if let Some(gap) = gap {
            inner = inner.gap(gap);
        }
        if let Some(padding) = padding {
            inner = inner.padding(padding);
        }

        inner.into()
    }
}
