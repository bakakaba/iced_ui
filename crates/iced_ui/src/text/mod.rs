//! A styled text widget with heading-level constructors.
//!
//! [`Text`] wraps plain text content and renders it at a size
//! determined by the heading [`Level`]. All headings are bold.
//! The font size is a multiple of the renderer's default text size.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::text::Text;
//!
//! let title = Text::h1("Page Title");
//! let section = Text::h2("Section");
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text::Paragraph as _;
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::{Text as AdvancedText, text};
use iced::alignment;
use iced::mouse;
use iced::{Color, Element, Length, Pixels, Rectangle, Size};

use crate::FontSizeBase;

/// The heading level, determining font size relative to the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Level {
    /// 3x default font size, bold.
    #[default]
    H1,
    /// 2x default font size, bold.
    H2,
    /// 1.5x default font size, bold.
    H3,
    /// 1.2x default font size, bold.
    H4,
    /// 1x default font size, bold.
    H5,
}

impl Level {
    /// Returns the font size multiplier for this level.
    pub fn multiplier(self) -> f32 {
        match self {
            Self::H1 => 2.2,
            Self::H2 => 1.8,
            Self::H3 => 1.5,
            Self::H4 => 1.2,
            Self::H5 => 1.0,
        }
    }
}

/// A styled text widget with heading-level support.
///
/// Renders plain text content at a size determined by the heading
/// [`Level`]. All headings use a bold font weight.
pub struct Text<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    content: String,
    level: Level,
    color: Option<Color>,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Text<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a new text widget with the given content and level.
    pub fn new(content: impl Into<String>, level: Level) -> Self {
        Self {
            content: content.into(),
            level,
            color: None,
            class: Theme::default(),
        }
    }

    /// Creates an H1 heading (3x default size, bold).
    pub fn h1(content: impl Into<String>) -> Self {
        Self::new(content, Level::H1)
    }

    /// Creates an H2 heading (2x default size, bold).
    pub fn h2(content: impl Into<String>) -> Self {
        Self::new(content, Level::H2)
    }

    /// Creates an H3 heading (1.5x default size, bold).
    pub fn h3(content: impl Into<String>) -> Self {
        Self::new(content, Level::H3)
    }

    /// Creates an H4 heading (1.2x default size, bold).
    pub fn h4(content: impl Into<String>) -> Self {
        Self::new(content, Level::H4)
    }

    /// Creates an H5 heading (1x default size, bold).
    pub fn h5(content: impl Into<String>) -> Self {
        Self::new(content, Level::H5)
    }

    /// Sets an explicit text color, overriding the style.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the style class for this text.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Returns the resolved font size in pixels.
    ///
    /// Uses the theme's text size when available (in `draw`), or falls
    /// back to the renderer's default size (in `layout` where theme is
    /// not available).
    fn font_size_from_base(&self, base: f32) -> f32 {
        base * self.level.multiplier()
    }
}

/// Returns a bold variant of the renderer's default font.
fn bold_font() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::default()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Text<'a, Theme>
where
    Theme: Catalog + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let font_size = self.font_size_from_base(renderer.default_size().0);
        let font = bold_font();
        let limits = limits.width(Length::Shrink).height(Length::Shrink);
        let max = limits.max();

        let paragraph = Renderer::Paragraph::with_text(text::Text {
            content: &self.content,
            bounds: Size::new(max.width, f32::INFINITY),
            size: Pixels(font_size),
            line_height: text::LineHeight::default(),
            font,
            align_x: alignment::Horizontal::Left.into(),
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        });

        let size = Size::new(paragraph.min_width(), paragraph.min_height());
        layout::Node::new(limits.resolve(Length::Shrink, Length::Shrink, size))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let theme_style = Catalog::style(theme, &self.class);
        let color = self.color.or(theme_style.color).unwrap_or(style.text_color);

        let bounds = layout.bounds();
        let font_size = self.font_size_from_base(theme.text_size());
        let font = bold_font();

        renderer.fill_text(
            AdvancedText {
                content: self.content.clone(),
                bounds: Size::new(bounds.width, bounds.height),
                size: Pixels(font_size),
                line_height: text::LineHeight::default(),
                font,
                align_x: alignment::Horizontal::Left.into(),
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
            iced::Point::new(bounds.x, bounds.y),
            color,
            bounds,
        );
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, Message, Theme, Renderer> From<Text<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + FontSizeBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
    fn from(text: Text<'a, Theme>) -> Self {
        Element::new(text)
    }
}
