//! Title bar with navigation icon, title, and action icons.
//!
//! # Example
//!
//! ```ignore
//! use iced::widget::text;
//! use iced_ui::top_app_bar::TopAppBar;
//!
//! let bar = TopAppBar::new("My App")
//!     .navigation_icon(my_back_button.into())
//!     .action(my_search_icon.into())
//!     .action(my_menu_icon.into());
//! ```

mod style;

pub use style::{Catalog, Style, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Pixels, Rectangle, Size};

use crate::FontSizeBase;

/// Height of the small top app bar in logical pixels (MD3 spec).
const BAR_HEIGHT: f32 = 64.0;

/// Width/height allocated for each icon cell (nav icon or action).
const ICON_CELL: f32 = 48.0;

/// Renders a fixed-height bar with an optional navigation icon on the
/// left, a title in the middle, and optional action icons on the
/// right.
pub struct TopAppBar<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    title: String,
    navigation_icon: Option<Element<'a, Message, Theme, Renderer>>,
    actions: Vec<Element<'a, Message, Theme, Renderer>>,
    class: Theme::Class<'a>,
    width: Length,
}

impl<'a, Message, Theme, Renderer> TopAppBar<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    /// Creates a new [`TopAppBar`] with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            navigation_icon: None,
            actions: Vec::new(),
            class: Theme::default(),
            width: Length::Fill,
        }
    }

    /// Sets the navigation icon element (displayed on the left).
    pub fn navigation_icon(
        mut self,
        icon: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.navigation_icon = Some(icon.into());
        self
    }

    /// Appends an action icon element (displayed on the right).
    pub fn action(mut self, action: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the width of the app bar.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Total number of child elements (nav icon + actions).
    fn child_count(&self) -> usize {
        self.navigation_icon.is_some() as usize + self.actions.len()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TopAppBar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<()>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(())
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(self.child_count());
        if let Some(nav) = &self.navigation_icon {
            children.push(Tree::new(nav));
        }
        for action in &self.actions {
            children.push(Tree::new(action));
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut refs: Vec<&Element<'_, Message, Theme, Renderer>> =
            Vec::with_capacity(self.child_count());
        if let Some(nav) = &self.navigation_icon {
            refs.push(nav);
        }
        for action in &self.actions {
            refs.push(action);
        }
        tree.diff_children(&refs);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(BAR_HEIGHT))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let total_width = limits.resolve(self.width, Length::Shrink, Size::ZERO).width;
        let mut children = Vec::with_capacity(self.child_count());
        let mut child_idx = 0;

        // Navigation icon (left).
        if let Some(nav) = &mut self.navigation_icon {
            let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_CELL, ICON_CELL));
            let mut node =
                nav.as_widget_mut()
                    .layout(&mut tree.children[child_idx], renderer, &icon_limits);
            let icon_y = (BAR_HEIGHT - node.size().height) / 2.0;
            let icon_x = (ICON_CELL - node.size().width) / 2.0;
            node = node.move_to(iced::Point::new(icon_x, icon_y));
            children.push(node);
            child_idx += 1;
        }

        // Actions (right) — layout from right to left.
        let actions_total_width = ICON_CELL * self.actions.len() as f32;
        let actions_start_x = total_width - actions_total_width;

        for action in &mut self.actions {
            let action_idx = child_idx - self.navigation_icon.is_some() as usize;
            let action_x = actions_start_x + ICON_CELL * action_idx as f32;
            let icon_limits = layout::Limits::new(Size::ZERO, Size::new(ICON_CELL, ICON_CELL));
            let mut node = action.as_widget_mut().layout(
                &mut tree.children[child_idx],
                renderer,
                &icon_limits,
            );
            let icon_y = (BAR_HEIGHT - node.size().height) / 2.0;
            let icon_x = (ICON_CELL - node.size().width) / 2.0;
            node = node.move_to(iced::Point::new(action_x + icon_x, icon_y));
            children.push(node);
            child_idx += 1;
        }

        layout::Node::with_children(Size::new(total_width, BAR_HEIGHT), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let style = Catalog::style(theme, &self.class);
        let bounds = layout.bounds();

        // Draw background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        // Draw children (nav icon + actions).
        let child_style = renderer::Style {
            text_color: style.icon_color,
        };

        let mut child_idx = 0;
        let mut layout_children = layout.children();

        if let Some(nav) = &self.navigation_icon {
            if let Some(child_layout) = layout_children.next() {
                nav.as_widget().draw(
                    &tree.children[child_idx],
                    renderer,
                    theme,
                    &child_style,
                    child_layout,
                    cursor,
                    viewport,
                );
            }
            child_idx += 1;
        }

        for action in &self.actions {
            if let Some(child_layout) = layout_children.next() {
                action.as_widget().draw(
                    &tree.children[child_idx],
                    renderer,
                    theme,
                    &child_style,
                    child_layout,
                    cursor,
                    viewport,
                );
            }
            child_idx += 1;
        }

        // Draw title text.
        // Title occupies the space between nav icon and actions.
        let title_x = if self.navigation_icon.is_some() {
            ICON_CELL
        } else {
            16.0
        };
        let actions_total_width = ICON_CELL * self.actions.len() as f32;
        let title_width = bounds.width - title_x - actions_total_width;

        if title_width > 0.0 {
            let title_bounds = Size::new(title_width, BAR_HEIGHT);
            let text = iced::advanced::text::Text {
                content: self.title.clone(),
                bounds: title_bounds,
                size: Pixels(theme.text_size() * 1.25),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::alignment::Horizontal::Left.into(),
                align_y: iced::alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::None,
            };

            let title_style = renderer::Style {
                text_color: style.title_color,
            };
            let _ = &title_style;

            renderer.fill_text(
                text,
                iced::Point::new(bounds.x + title_x, bounds.center_y()),
                style.title_color,
                bounds,
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
        let mut child_idx = 0;
        let mut layout_children = layout.children();

        if let Some(nav) = &mut self.navigation_icon {
            if let Some(child_layout) = layout_children.next() {
                nav.as_widget_mut().update(
                    &mut tree.children[child_idx],
                    event,
                    child_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
            child_idx += 1;
        }

        for action in &mut self.actions {
            if let Some(child_layout) = layout_children.next() {
                action.as_widget_mut().update(
                    &mut tree.children[child_idx],
                    event,
                    child_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
            child_idx += 1;
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut child_idx = 0;
        let mut layout_children = layout.children();

        if let Some(nav) = &self.navigation_icon {
            if let Some(child_layout) = layout_children.next() {
                let interaction = nav.as_widget().mouse_interaction(
                    &tree.children[child_idx],
                    child_layout,
                    cursor,
                    viewport,
                    renderer,
                );
                if interaction != mouse::Interaction::default() {
                    return interaction;
                }
            }
            child_idx += 1;
        }

        for action in &self.actions {
            if let Some(child_layout) = layout_children.next() {
                let interaction = action.as_widget().mouse_interaction(
                    &tree.children[child_idx],
                    child_layout,
                    cursor,
                    viewport,
                    renderer,
                );
                if interaction != mouse::Interaction::default() {
                    return interaction;
                }
            }
            child_idx += 1;
        }

        mouse::Interaction::default()
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut child_idx = 0;
        let mut layout_children = layout.children();

        if let Some(nav) = &mut self.navigation_icon {
            if let Some(child_layout) = layout_children.next() {
                nav.as_widget_mut().operate(
                    &mut tree.children[child_idx],
                    child_layout,
                    renderer,
                    operation,
                );
            }
            child_idx += 1;
        }

        for action in &mut self.actions {
            if let Some(child_layout) = layout_children.next() {
                action.as_widget_mut().operate(
                    &mut tree.children[child_idx],
                    child_layout,
                    renderer,
                    operation,
                );
            }
            child_idx += 1;
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TopAppBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + FontSizeBase + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(bar: TopAppBar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(bar)
    }
}
