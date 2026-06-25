//! Compact, pill-shaped interactive element for actions, filtering, or
//! representing input.
//!
//! A chip is always rendered as a full pill (its corner radius is half
//! its height). By default it is a transparent, outlined pill; giving
//! it a [`ChipColor`] fills the pill with the resolved color and uses a
//! readable contrast for the label. Chips come in three
//! [`sizes`](ChipSize) — one step denser than [`Button`](crate::Button)
//! — and the chip drives its label font size from the chosen size.
//!
//! A chip is interactive only when it has a handler: [`on_toggle`] makes
//! the body clickable, and [`on_remove`] adds a circular "x" button at
//! the trailing end. A chip with neither handler is a static token and
//! shows no pointer or hover affordance.
//!
//! [`on_toggle`]: Chip::on_toggle
//! [`on_remove`]: Chip::on_remove
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::chip::{Chip, ChipColor, ChipSize};
//!
//! // Outlined pill (default).
//! let plain = Chip::new("Add event").on_toggle(Message::Add);
//!
//! // Filled, large, removable pill.
//! let filled = Chip::new("Vegetarian")
//!     .color(ChipColor::Primary)
//!     .size(ChipSize::Lg)
//!     .on_toggle(Message::Toggle)
//!     .on_remove(Message::Remove);
//! ```

mod style;

pub use style::{Catalog, ChipColor, ChipSize, Status, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph, Text};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::alignment;
use iced::mouse;
use iced::{Element, Event, Font, Length, Pixels, Point, Rectangle, Size};

use std::cell::Cell;

use crate::{FontSizeBase, SpacingBase};

/// Maximum icon size in logical pixels.
const ICON_MAX: f32 = 18.0;
/// Gap between the label and an icon in logical pixels.
const ICON_GAP: f32 = 8.0;

/// Returns the remove-button glyph and font.
#[cfg(feature = "lucide-icons")]
fn remove_icon() -> (String, Font) {
    use crate::icons::FONT;
    (char::from(lucide_icons::Icon::X).to_string(), FONT)
}

/// Returns the remove-button glyph and font (Unicode fallback).
#[cfg(not(feature = "lucide-icons"))]
fn remove_icon() -> (String, Font) {
    ("\u{00D7}".to_string(), Font::default())
}

/// Internal state.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_hovered: bool,
    is_pressed: bool,
}

/// Chip widget.
pub struct Chip<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + text::Renderer,
{
    label: String,
    leading_icon: Option<Element<'a, Message, Theme, Renderer>>,
    color: Option<ChipColor>,
    size: ChipSize,
    on_toggle: Option<Message>,
    on_remove: Option<Message>,
    enabled: bool,
    class: Theme::Class<'a>,
    /// Cache of the theme tokens resolved in `draw` and read back in
    /// `layout` (which has no `&Theme`): `(base_text_size, spacing)`.
    /// Seeded with the theme defaults so the first layout pass is
    /// sensible.
    theme_cache: Cell<(f32, u8)>,
}

impl<'a, Message, Theme, Renderer> Chip<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + text::Renderer,
{
    /// Creates a chip with the given label.
    ///
    /// The chip starts as a transparent, outlined pill at the default
    /// [`size`](ChipSize::Md). Use [`color`](Self::color) to fill it.
    /// The label font size is driven by the chip's [`size`](Self::size).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            leading_icon: None,
            color: None,
            size: ChipSize::default(),
            on_toggle: None,
            on_remove: None,
            enabled: true,
            class: Theme::default(),
            theme_cache: Cell::new((
                crate::Theme::DEFAULT_TEXT_SIZE,
                crate::Theme::DEFAULT_SPACING,
            )),
        }
    }

    /// Sets the color token used to fill the pill.
    ///
    /// When set, the pill is filled with the resolved color and the
    /// label uses its readable contrast. When unset (the default), the
    /// chip renders as a transparent, outlined pill.
    pub fn color(mut self, color: impl Into<ChipColor>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the chip size. Defaults to [`ChipSize::Md`].
    ///
    /// The size drives both the pill height and the label font size.
    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }

    /// Sets a leading icon element.
    pub fn icon(mut self, icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Sets the message emitted when the chip body is clicked.
    ///
    /// A chip with an `on_toggle` handler is interactive: it shows a
    /// pointer cursor and a hover/press highlight.
    pub fn on_toggle(mut self, message: Message) -> Self {
        self.on_toggle = Some(message);
        self
    }

    /// Sets the message emitted when the chip body is clicked, if
    /// `Some`.
    pub fn on_toggle_maybe(mut self, message: Option<Message>) -> Self {
        self.on_toggle = message;
        self
    }

    /// Sets the message emitted when the chip's remove button is
    /// clicked.
    ///
    /// When set, the chip renders a small circular "x" button at its
    /// trailing end; clicking it emits this message.
    pub fn on_remove(mut self, message: Message) -> Self {
        self.on_remove = Some(message);
        self
    }

    /// Enables or disables the chip.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Returns the leading icon element, if any, as the chip's only
    /// child widget.
    fn icons(&self) -> impl Iterator<Item = &Element<'a, Message, Theme, Renderer>> {
        self.leading_icon.iter()
    }

    fn icons_mut(&mut self) -> impl Iterator<Item = &mut Element<'a, Message, Theme, Renderer>> {
        self.leading_icon.iter_mut()
    }

    /// Whether the chip body responds to clicks (pointer + hover/press).
    fn body_interactive(&self) -> bool {
        self.enabled && self.on_toggle.is_some()
    }

    /// Whether the chip shows a remove button.
    fn removable(&self) -> bool {
        self.on_remove.is_some()
    }

    /// The diameter of the remove button for the given font size.
    fn remove_diameter(font_size: f32) -> f32 {
        font_size
    }

    /// The circular remove button's bounds within the given chip
    /// bounds, right-aligned inside the trailing horizontal padding and
    /// vertically centered.
    fn remove_bounds(chip_bounds: Rectangle, h_padding: f32, diameter: f32) -> Rectangle {
        Rectangle {
            x: chip_bounds.x + chip_bounds.width - h_padding - diameter,
            y: chip_bounds.y + (chip_bounds.height - diameter) / 2.0,
            width: diameter,
            height: diameter,
        }
    }

    /// Measures the label's natural size at the given font size using
    /// the renderer's text shaper. Used both to lay out the pill and to
    /// center the label when drawing.
    fn measure_label(&self, renderer: &Renderer, font_size: f32) -> Size
    where
        Renderer: text::Renderer,
    {
        let paragraph = Renderer::Paragraph::with_text(Text {
            content: self.label.as_str(),
            bounds: Size::INFINITE,
            size: Pixels(font_size),
            line_height: text::LineHeight::Relative(1.0),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        });
        paragraph.min_bounds()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Chip<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + SpacingBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        // Only the icon elements are part of the widget tree; the label
        // is text rendered directly by the chip.
        self.icons().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let refs: Vec<&Element<'_, Message, Theme, Renderer>> = self.icons().collect();
        tree.diff_children(&refs);
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
        let (base_text_size, spacing) = self.theme_cache.get();
        let font_size = self.size.font_size(base_text_size);
        // Padding derived from theme spacing, scaled per size.
        let (h_pad_token, v_pad_token) = self.size.padding();
        let h_padding = h_pad_token.resolve(spacing);
        let v_padding = v_pad_token.resolve(spacing);

        // Leading icon node, laid out and positioned (vertically
        // centered) once the pill height is known. The label and remove
        // button are self-rendered, not layout children.
        let leading_node = self.leading_icon.as_mut().map(|icon| {
            let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_MAX, ICON_MAX));
            icon.as_widget_mut()
                .layout(&mut tree.children[0], renderer, &icon_limits)
        });
        let leading_width = leading_node
            .as_ref()
            .map(|n| n.size().width + ICON_GAP)
            .unwrap_or(0.0);

        // Measure the label using the renderer's text shaper.
        let label_size = self.measure_label(renderer, font_size);
        let label_width = label_size.width;

        // Trailing remove button reserves a circle plus a gap.
        let remove_width = if self.removable() {
            ICON_GAP + Self::remove_diameter(font_size)
        } else {
            0.0
        };

        // The pill height fits the tallest content (label or icon) plus
        // vertical padding above and below.
        let leading_height = leading_node
            .as_ref()
            .map(|n| n.size().height)
            .unwrap_or(0.0);
        let content_height = font_size.max(leading_height).max(label_size.height);
        let chip_height = content_height + v_padding * 2.0;

        let total_width = h_padding + leading_width + label_width + remove_width + h_padding;
        let _ = limits;

        let mut positioned = Vec::new();
        if let Some(node) = leading_node {
            let h = node.size().height;
            positioned.push(node.move_to(Point::new(h_padding, (chip_height - h) / 2.0)));
        }

        layout::Node::with_children(Size::new(total_width, chip_height), positioned)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let status = if !self.enabled {
            Status::Disabled
        } else if self.body_interactive() && state.is_pressed {
            Status::Pressed
        } else if self.body_interactive() && state.is_hovered {
            Status::Hovered
        } else {
            Status::Active
        };

        let mut chip_style = Catalog::style(theme, &self.class, self.color, status);
        // Force a full pill: the corner radius is half the chip height,
        // regardless of theme roundness or any custom style closure.
        chip_style.border.radius = (bounds.height / 2.0).into();

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: chip_style.border,
                shadow: chip_style.shadow,
                ..renderer::Quad::default()
            },
            chip_style
                .background
                .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
        );

        let font_size = self.size.font_size(theme.text_size());
        // Cache the theme tokens so the next `layout` (which has no
        // `&Theme`) resolves the same font size and padding.
        self.theme_cache.set((theme.text_size(), theme.spacing()));
        let (h_pad_token, _) = self.size.padding();
        let h_padding = h_pad_token.resolve(theme.spacing());

        // Draw leading icon (the chip's only layout child) if present.
        let mut child_layouts = layout.children();
        let mut label_x = bounds.x + h_padding;
        if let Some(icon) = &self.leading_icon {
            let icon_layout = child_layouts.next().unwrap();
            icon.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &renderer::Style {
                    text_color: chip_style.text_color,
                },
                icon_layout,
                cursor,
                viewport,
            );
            label_x = icon_layout.bounds().x + icon_layout.bounds().width + ICON_GAP;
        }

        // The label region ends before the remove button (if any).
        let label_region_right = if self.removable() {
            bounds.x + bounds.width - h_padding - Self::remove_diameter(font_size) - ICON_GAP
        } else {
            bounds.x + bounds.width - h_padding
        };

        // Draw the label text, centered within its region, using the
        // renderer-measured width so it never wraps or clips.
        let label_width = self.measure_label(renderer, font_size).width;
        let region_center = (label_x + label_region_right) / 2.0;
        renderer.fill_text(
            Text {
                content: self.label.clone(),
                bounds: Size::new(label_width, bounds.height),
                size: Pixels(font_size),
                line_height: text::LineHeight::Relative(1.0),
                font: renderer.default_font(),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
            Point::new(region_center, bounds.y + bounds.height / 2.0),
            chip_style.text_color,
            *viewport,
        );

        // Draw the remove button (circle + "x") if present.
        if self.removable() {
            let diameter = Self::remove_diameter(font_size);
            let circle = Self::remove_bounds(bounds, h_padding, diameter);
            // Subtle translucent circle derived from the label color.
            renderer.fill_quad(
                renderer::Quad {
                    bounds: circle,
                    border: iced::Border {
                        radius: (diameter / 2.0).into(),
                        ..iced::Border::default()
                    },
                    ..renderer::Quad::default()
                },
                iced::Background::Color(iced::Color {
                    a: 0.12,
                    ..chip_style.text_color
                }),
            );

            let (glyph, font) = remove_icon();
            renderer.fill_text(
                Text {
                    content: glyph,
                    bounds: Size::new(diameter, diameter),
                    size: Pixels(diameter * 0.75),
                    line_height: text::LineHeight::Relative(1.0),
                    font,
                    align_x: text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                },
                Point::new(circle.center_x(), circle.center_y()),
                chip_style.text_color,
                *viewport,
            );
        }
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
        // Forward to icon children.
        for (tree_idx, (icon, child_layout)) in self.icons_mut().zip(layout.children()).enumerate()
        {
            icon.as_widget_mut().update(
                &mut tree.children[tree_idx],
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        // A chip only reacts to clicks when enabled and has at least
        // one handler (toggle body and/or remove button).
        if !self.enabled || (self.on_toggle.is_none() && self.on_remove.is_none()) {
            return;
        }

        let (base_text_size, spacing) = self.theme_cache.get();
        let font_size = self.size.font_size(base_text_size);
        let (h_pad_token, _) = self.size.padding();
        let h_padding = h_pad_token.resolve(spacing);

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let is_over = cursor.is_over(bounds);

        // Whether the cursor is over the remove button specifically.
        let over_remove = |cursor: mouse::Cursor| -> bool {
            if !self.removable() {
                return false;
            }
            let circle = Self::remove_bounds(bounds, h_padding, Self::remove_diameter(font_size));
            cursor.position().is_some_and(|p| circle.contains(p))
        };

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Only the toggle body shows a hover highlight.
                state.is_hovered = is_over && self.on_toggle.is_some();
                if !is_over {
                    state.is_pressed = false;
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                state.is_pressed = true;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_pressed && is_over {
                    if over_remove(cursor) {
                        if let Some(msg) = self.on_remove.clone() {
                            shell.publish(msg);
                        }
                    } else if let Some(msg) = self.on_toggle.clone() {
                        shell.publish(msg);
                    }
                }
                state.is_pressed = false;
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if !self.enabled || !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::default();
        }

        // A clickable body (on_toggle) makes the whole pill a pointer.
        // A remove-only chip shows the pointer only over the x button.
        if self.on_toggle.is_some() {
            mouse::Interaction::Pointer
        } else if self.removable() {
            let (base_text_size, spacing) = self.theme_cache.get();
            let font_size = self.size.font_size(base_text_size);
            let (h_pad_token, _) = self.size.padding();
            let h_padding = h_pad_token.resolve(spacing);
            let circle =
                Self::remove_bounds(layout.bounds(), h_padding, Self::remove_diameter(font_size));
            if cursor.position().is_some_and(|p| circle.contains(p)) {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for (tree_idx, (icon, child_layout)) in self.icons_mut().zip(layout.children()).enumerate()
        {
            icon.as_widget_mut().operate(
                &mut tree.children[tree_idx],
                child_layout,
                renderer,
                operation,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Chip<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + SpacingBase + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = iced::Font> + 'a,
{
    fn from(chip: Chip<'a, Message, Theme, Renderer>) -> Self {
        Element::new(chip)
    }
}
