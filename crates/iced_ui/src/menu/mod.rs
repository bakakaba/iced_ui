//! A horizontal application menu bar with dropdowns, nested submenus,
//! keyboard shortcuts and a pluggable style.
//!
//! See [`MenuBar`] for the widget, [`Item`] and [`Menu`] for its
//! entries, and [`shortcuts`] plus [`KeyBinding`] for key-binding
//! support.

mod item;
mod overlay;
mod shortcut;
mod style;
mod trigger;

use std::cell::Cell;

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay as advanced_overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::widget::text as iced_text;
use iced::{Element, Event, Font, Length, Padding, Pixels, Point, Rectangle, Size, Vector};

pub use item::{Entry, Icon, Item, Menu, Separator};
pub use shortcut::{KeyBinding, shortcuts};
pub use style::{Catalog, Style, StyleFn, default};
pub use trigger::MenuButton;

use crate::button::{self, Button, Variant};
use crate::{PaddingSource, RoundnessBase, SpacingBase};

use item::collect_shortcuts;
use overlay::{MenuOverlay, Metrics};

/// Internal widget state stored in the widget tree.
#[derive(Debug)]
pub(crate) struct State {
    /// Path of currently-open menus. `None` means closed.
    pub(crate) open_path: Option<Vec<usize>>,
    /// Index of the top-level bar label that is visually "active"
    /// (currently open or pressed).
    pub(crate) bar_active: Option<usize>,
    /// Index of the top-level bar label currently under the cursor
    /// (when the bar is closed).
    pub(crate) bar_hover: Option<usize>,
    /// Rows of menus that the cursor is currently hovering, per depth.
    pub(crate) hover_path: Vec<Option<usize>>,
    /// Last-resolved `Style::spacing`, cached for use in overlay.
    pub(crate) spacing_cache: Cell<f32>,
    /// Last-resolved padding (in logical pixels), cached for overlay.
    pub(crate) padding_cache: Cell<Padding>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open_path: None,
            bar_active: None,
            bar_hover: None,
            hover_path: Vec::new(),
            spacing_cache: Cell::new(8.0),
            padding_cache: Cell::new(Padding::new(4.0)),
        }
    }
}

impl State {
    pub(crate) fn close(&mut self) {
        self.open_path = None;
        self.hover_path.clear();
    }
}

/// A horizontal application menu bar.
///
/// See the [module-level docs](crate::menu) for an overview, and
/// [`MenuBar::shortcuts`] for how to wire up keyboard shortcuts.
pub struct MenuBar<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog + button::Catalog + SpacingBase + RoundnessBase + iced::widget::text::Catalog,
    Renderer: renderer::Renderer + text::Renderer<Font = Font>,
{
    menus: Vec<Menu<Message>>,
    triggers: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    padding: PaddingSource,
    text_size: Option<Pixels>,
    font: Option<Font>,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme:
        Catalog + button::Catalog + SpacingBase + RoundnessBase + iced::widget::text::Catalog + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = Font> + 'a,
{
    /// Creates an empty [`MenuBar`].
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            triggers: Vec::new(),
            width: Length::Fill,
            padding: PaddingSource::from(crate::Space::sx(1.0)),
            text_size: None,
            font: None,
            class: <Theme as Catalog>::default(),
        }
    }

    /// Creates a [`MenuBar`] pre-populated with the given top-level menus.
    pub fn with_menus(menus: Vec<Menu<Message>>) -> Self {
        let mut bar = Self::new();
        for menu in menus {
            bar = bar.push(menu);
        }
        bar
    }

    /// Appends a top-level [`Menu`] to this bar.
    pub fn push(mut self, menu: Menu<Message>) -> Self {
        let label = menu.label.clone();
        let trigger: Element<'a, Message, Theme, Renderer> = Button::new(iced_text(label))
            .variant(Variant::Ghost)
            .size(button::ButtonSize::Sm)
            .color(button::ButtonColor::Foreground)
            .into();
        self.triggers.push(trigger);
        self.menus.push(menu);
        self
    }

    /// Sets the width of the bar. Defaults to [`Length::Fill`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the outer padding around the bar's row of labels.
    /// Defaults to [`Space::sx(1.0)`](crate::Space::sx), which
    /// resolves to `4.0` logical pixels at the default
    /// [`spacing`].
    ///
    /// Accepts a [`Space`](crate::Space), an [`iced::Padding`] or a
    /// raw `[f32; 2]` (absolute).
    ///
    /// [`spacing`]: crate::Theme::spacing
    pub fn padding(mut self, padding: impl Into<PaddingSource>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size used for menu items in dropdowns.
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the font used for menu items in dropdowns.
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }
}

impl<'a, Message, Renderer> MenuBar<'a, Message, crate::Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = Font> + 'a,
{
    /// Sets the style of the bar using a `Fn(&Theme) -> Style` closure.
    ///
    /// This is available when `Theme = `[`crate::Theme`].
    pub fn style(mut self, style: impl Fn(&crate::Theme) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<Message: Clone + 'static, Theme, Renderer> MenuBar<'_, Message, Theme, Renderer>
where
    Theme: Catalog + button::Catalog + SpacingBase + RoundnessBase + iced::widget::text::Catalog,
    Renderer: renderer::Renderer + text::Renderer<Font = Font>,
{
    /// Returns the list of all `(KeyBinding, Message)` pairs declared
    /// on items within this bar (including inside nested submenus).
    ///
    /// Pass the returned vec to [`shortcuts`] to produce a
    /// [`Subscription`](iced::Subscription) that dispatches the
    /// matching message whenever the user presses one of the
    /// combinations.
    pub fn shortcuts(&self) -> Vec<(KeyBinding, Message)> {
        collect_shortcuts(&self.menus)
    }
}

impl<Message, Theme, Renderer> Default for MenuBar<'_, Message, Theme, Renderer>
where
    Message: Clone + 'static,
    Theme: Catalog
        + button::Catalog
        + SpacingBase
        + RoundnessBase
        + iced::widget::text::Catalog
        + 'static,
    Renderer: renderer::Renderer + text::Renderer<Font = Font> + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MenuBar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme:
        Catalog + button::Catalog + SpacingBase + RoundnessBase + iced::widget::text::Catalog + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = Font> + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let initial_padding = self.padding.resolve(crate::Theme::DEFAULT_SPACING);
        let state = State {
            padding_cache: Cell::new(initial_padding),
            ..State::default()
        };
        tree::State::new(state)
    }

    fn children(&self) -> Vec<Tree> {
        self.triggers.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.triggers.iter().collect::<Vec<_>>());
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let spacing = state.spacing_cache.get();
        let padding = state.padding_cache.get();
        let active = state.bar_active;

        // Rebuild triggers with the correct focused state.
        self.triggers = self
            .menus
            .iter()
            .enumerate()
            .map(|(i, menu)| {
                Button::new(iced_text(menu.label.clone()))
                    .variant(Variant::Ghost)
                    .size(button::ButtonSize::Sm)
                    .color(button::ButtonColor::Foreground)
                    .focused(active == Some(i))
                    .into()
            })
            .collect();

        // Diff children to match rebuilt triggers.
        tree.diff_children(&self.triggers.iter().collect::<Vec<_>>());

        // Layout each trigger button.
        let child_limits = layout::Limits::new(Size::ZERO, limits.max());
        let mut children_nodes = Vec::with_capacity(self.triggers.len());
        let mut x = padding.left;
        let trigger_count = self.triggers.len();

        for (i, trigger) in self.triggers.iter_mut().enumerate() {
            let mut node =
                trigger
                    .as_widget_mut()
                    .layout(&mut tree.children[i], renderer, &child_limits);
            node = node.move_to(Point::new(x, padding.top));
            x += node.size().width;
            if i + 1 < trigger_count {
                x += spacing;
            }
            children_nodes.push(node);
        }

        x += padding.right;

        let bar_height = children_nodes
            .iter()
            .map(|n| n.size().height)
            .fold(0.0_f32, f32::max)
            + padding.top
            + padding.bottom;

        let max_width = limits.max().width;
        let width = match self.width {
            Length::Shrink => x.min(max_width),
            _ => max_width,
        };

        layout::Node::with_children(Size::new(width, bar_height), children_nodes)
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
        let state = tree.state.downcast_ref::<State>();
        let menu_style = Catalog::style(theme, &self.class);
        state.spacing_cache.set(menu_style.spacing);
        let padding = self.padding.resolve(theme.spacing());
        state.padding_cache.set(padding);
        let bounds = layout.bounds();

        // Draw bar background.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: menu_style.bar_border,
                ..renderer::Quad::default()
            },
            menu_style.bar_background,
        );

        // Draw child trigger buttons.
        for ((i, child_layout), child_tree) in
            layout.children().enumerate().zip(tree.children.iter())
        {
            self.triggers[i].as_widget().draw(
                child_tree,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
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
        let state = tree.state.downcast_mut::<State>();

        // Compute child bounds for hit-testing.
        let child_bounds: Vec<Rectangle> = layout.children().map(|l| l.bounds()).collect();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new_hover = cursor
                    .position()
                    .and_then(|pos| child_bounds.iter().position(|b| b.contains(pos)));

                // If a menu is already open and the user hovers a
                // different trigger, switch to that menu immediately.
                if state.open_path.is_some()
                    && let Some(i) = new_hover
                    && state.bar_active != Some(i)
                {
                    state.open_path = Some(vec![i]);
                    state.bar_active = Some(i);
                    state.hover_path = vec![Some(i)];
                    shell.request_redraw();
                }

                if state.bar_hover != new_hover {
                    state.bar_hover = new_hover;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) if state.bar_hover.is_some() => {
                state.bar_hover = None;
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    for (i, b) in child_bounds.iter().enumerate() {
                        if b.contains(pos) {
                            if state.open_path == Some(vec![i]) {
                                // Toggle off.
                                state.close();
                                state.bar_active = None;
                            } else {
                                state.open_path = Some(vec![i]);
                                state.bar_active = Some(i);
                                state.hover_path = vec![Some(i)];
                            }
                            shell.capture_event();
                            shell.request_redraw();
                            return;
                        }
                    }
                }
            }
            _ => {}
        }

        // Forward events to child triggers for hover styling.
        for ((i, child_layout), child_tree) in
            layout.children().enumerate().zip(tree.children.iter_mut())
        {
            self.triggers[i].as_widget_mut().update(
                child_tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        for ((i, child_layout), child_tree) in
            layout.children().enumerate().zip(tree.children.iter())
        {
            let interaction = self.triggers[i].as_widget().mouse_interaction(
                child_tree,
                child_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::default() {
                return interaction;
            }
        }

        mouse::Interaction::default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced_overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        state.open_path.as_ref()?;

        let bar_bounds = layout.bounds() + translation;
        let bar_label_bounds: Vec<Rectangle> = layout
            .children()
            .map(|l| l.bounds() + translation)
            .collect();

        let spacing = state.spacing_cache.get();
        let padding = state.padding_cache.get();
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let metrics = Metrics::new(text_size, padding, spacing);

        let overlay = MenuOverlay {
            menus: &mut self.menus,
            state,
            bar_bounds,
            bar_label_bounds,
            metrics,
            font,
            style_fn: &self.class,
            _renderer: std::marker::PhantomData,
        };

        Some(advanced_overlay::Element::new(Box::new(overlay)))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        for ((i, child_layout), child_tree) in
            layout.children().enumerate().zip(tree.children.iter_mut())
        {
            self.triggers[i]
                .as_widget_mut()
                .operate(child_tree, child_layout, renderer, operation);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<MenuBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme:
        Catalog + button::Catalog + SpacingBase + RoundnessBase + iced::widget::text::Catalog + 'a,
    Renderer: renderer::Renderer + text::Renderer<Font = Font> + 'a,
{
    fn from(bar: MenuBar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(bar)
    }
}
