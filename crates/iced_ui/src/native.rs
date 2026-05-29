//! The [`native`] escape-hatch function.
//!
//! Allows embedding an iced-native widget (one that expects
//! `iced::Theme`) directly in an `iced_ui::Theme` widget tree —
//! without requiring `iced_ui::Theme` to implement the widget's
//! `Catalog` trait.
//!
//! # When to use
//!
//! Use `native()` when iced introduces a new widget that `iced_ui`
//! hasn't added Catalog support for yet. This lets you keep building
//! your app without waiting for an `iced_ui` release.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::native;
//!
//! fn view(&self) -> iced::Element<'_, Message, iced_ui::Theme> {
//!     column![
//!         // iced_ui widget — works directly:
//!         iced_ui::Fab::new(text("+")).on_press(Message::Add),
//!
//!         // A hypothetical future iced widget without an iced_ui Catalog impl:
//!         native(some_future_iced_widget("hello").on_press(Message::Click)),
//!     ].into()
//! }
//! ```

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::Theme;

/// Embed a native iced widget in an [`iced_ui::Theme`](crate::Theme)
/// widget tree.
///
/// The wrapped widget will be styled using the underlying
/// [`iced::Theme`] from [`Theme::colors`](crate::Theme::colors). All
/// events (mouse, keyboard, touch) pass through transparently.
///
/// See the [module documentation](self) for usage details.
pub fn native<'a, Message: 'a>(
    content: impl Into<Element<'a, Message, iced::Theme>>,
) -> Element<'a, Message, Theme> {
    Element::new(Native {
        content: content.into(),
    })
}

struct Native<'a, Message> {
    content: Element<'a, Message, iced::Theme>,
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Native<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // Pass `&theme.colors` (the inner iced::Theme) to the wrapped
        // widget instead of the outer iced_ui::Theme.
        self.content.as_widget().draw(
            tree,
            renderer,
            &theme.colors,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut widget::Tree,
        _layout: Layout<'b>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, iced::Renderer>> {
        // Overlays from native widgets are not proxied. This is a
        // limitation of the escape hatch — widgets that produce overlays
        // (e.g., pick_list dropdowns) should use the auto-generated
        // Catalog impls instead.
        None
    }
}

impl<'a, Message: 'a> From<Native<'a, Message>> for Element<'a, Message, Theme> {
    fn from(native: Native<'a, Message>) -> Self {
        Element::new(native)
    }
}
