//! A hierarchical tree of expandable/collapsible nodes.
//!
//! Each [`Node`] wraps arbitrary content and may hold child nodes.
//! Whether a node is expanded is **controlled** by the caller via
//! [`Node::expanded`] — the widget does not track expansion state
//! internally. Clicking a node's disclosure indicator emits the
//! message produced by the [`Tree::on_toggle`] callback, carrying the
//! node's identifier so the parent application can flip the
//! corresponding flag.
//!
//! A node is laid out as `[disclosure indicator][content]`, and each
//! depth level adds one `indent` of leading space (see [`Tree::indent`]),
//! so a child's indicator/content block is shifted right by one indent
//! relative to its parent. The disclosure indicator defaults to a
//! chevron (a Lucide glyph when the `lucide-icons` feature is enabled,
//! otherwise a Unicode triangle) and can be overridden for both the
//! expanded and collapsed states via [`Tree::expanded_indicator`] and
//! [`Tree::collapsed_indicator`].
//!
//! See [`Tree`] for the widget and [`Catalog`]/[`ItemStyle`] for
//! styling.

mod style;

use std::cell::Cell;
use std::marker::PhantomData;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree as WidgetTree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::widget::{Space, text};
use iced::{Element, Event, Length, Padding, Rectangle, Size, Vector, mouse};

pub use style::{Catalog, ItemStyle, Status, StyleFn, default};

use crate::{PaddingSource, Space as ThemeSpace, SpacingBase};

/// A boxed factory that builds a disclosure indicator [`Element`] for a
/// given expansion state (`true` = expanded).
type IndicatorFn<'a, Message, Theme, Renderer> =
    Box<dyn Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a>;

/// A boxed callback mapping a toggled node's id to a message.
type ToggleFn<'a, Id, Message> = Box<dyn Fn(Id) -> Message + 'a>;

/// A single node in a [`Tree`].
///
/// Wraps arbitrary content, holds zero or more child [`Node`]s, and
/// carries an `Id` used to report toggle events. Whether the node is
/// expanded is controlled by the caller via [`Node::expanded`].
#[allow(missing_debug_implementations)]
pub struct Node<'a, Id, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    id: Id,
    content: Element<'a, Message, Theme, Renderer>,
    children: Vec<Node<'a, Id, Message, Theme, Renderer>>,
    expanded: bool,
}

impl<'a, Id, Message, Theme, Renderer> Node<'a, Id, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new leaf [`Node`] with the given identifier and
    /// content.
    pub fn new(id: Id, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            id,
            content: content.into(),
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Appends a child [`Node`].
    pub fn push(mut self, child: Node<'a, Id, Message, Theme, Renderer>) -> Self {
        self.children.push(child);
        self
    }

    /// Sets whether the node is expanded. Defaults to `false`.
    ///
    /// This flag is **controlled**: the widget renders the node's
    /// children only while it is `true`, but never mutates the flag
    /// itself. React to [`Tree::on_toggle`] to update it.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Returns `true` if the node has at least one child.
    fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Internal per-row interaction state.
#[derive(Debug, Clone, Copy, Default)]
struct RowState {
    is_hovered: bool,
    is_pressed: bool,
}

/// Internal widget state for a [`Tree`].
#[derive(Debug)]
struct TreeState {
    rows: Vec<RowState>,
    /// Cached outer-padding resolution for layout.
    padding_cache: Cell<Padding>,
    /// Cached per-level indentation resolution for layout.
    indent_cache: Cell<f32>,
    /// Cached inter-row spacing resolution for layout.
    spacing_cache: Cell<f32>,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            padding_cache: Cell::new(Padding::new(0.0)),
            indent_cache: Cell::new(0.0),
            spacing_cache: Cell::new(0.0),
        }
    }
}

/// A description of one visible row, produced by flattening the tree.
struct VisibleRow<Id> {
    /// Depth in the tree (root rows are depth 0).
    depth: usize,
    /// Whether this node has at least one child (and thus an
    /// interactive disclosure indicator).
    has_children: bool,
    /// Identifier of the node, used to report toggle events.
    id: Id,
}

/// A hierarchical, expandable tree widget.
///
/// Build a tree from [`Node`]s, each carrying an `Id`. Expansion is
/// controlled by the caller (see [`Node::expanded`]); wire
/// [`Tree::on_toggle`] to receive the id of the node whose disclosure
/// indicator was clicked.
///
/// # Example
///
/// ```no_run
/// use iced::widget::text;
/// use iced_ui::{tree, Theme};
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     Toggle(u32),
/// }
///
/// # fn _build(root_expanded: bool) -> iced::Element<'static, Message, Theme> {
/// tree::Tree::new()
///     .node(
///         tree::Node::new(0, text("Fruits"))
///             .expanded(root_expanded)
///             .push(tree::Node::new(1, text("Apple")))
///             .push(tree::Node::new(2, text("Banana"))),
///     )
///     .on_toggle(Message::Toggle)
///     .into()
/// # }
/// ```
#[allow(missing_debug_implementations)]
pub struct Tree<'a, Id, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    roots: Vec<Node<'a, Id, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    padding: PaddingSource,
    node_padding: PaddingSource,
    indent: ThemeSpace,
    spacing: ThemeSpace,
    on_toggle: Option<ToggleFn<'a, Id, Message>>,
    indicator: Option<IndicatorFn<'a, Message, Theme, Renderer>>,
    class: <Theme as Catalog>::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Id, Message, Theme, Renderer> Default for Tree<'a, Id, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Id, Message, Theme, Renderer> Tree<'a, Id, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new, empty [`Tree`].
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: PaddingSource::from(ThemeSpace::sx(1.0)),
            node_padding: PaddingSource::from(ThemeSpace::sx(1.0)),
            indent: ThemeSpace::sx(2.0),
            spacing: ThemeSpace::sx(0.0),
            on_toggle: None,
            indicator: None,
            class: <Theme as Catalog>::default(),
            _renderer: PhantomData,
        }
    }

    /// Appends a root [`Node`] to the tree.
    pub fn node(mut self, node: Node<'a, Id, Message, Theme, Renderer>) -> Self {
        self.roots.push(node);
        self
    }

    /// Sets the message-producing callback invoked when a node's
    /// disclosure indicator is clicked. The closure receives the
    /// clicked node's `Id`.
    pub fn on_toggle(mut self, on_toggle: impl Fn(Id) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    /// Sets the width of the [`Tree`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Tree`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the outer padding of the [`Tree`]. Defaults to
    /// [`Space::sx(1.0)`](crate::Space::sx).
    pub fn padding(mut self, padding: impl Into<PaddingSource>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the inner padding of each node row. Defaults to
    /// [`Space::sx(1.0)`](crate::Space::sx).
    pub fn node_padding(mut self, padding: impl Into<PaddingSource>) -> Self {
        self.node_padding = padding.into();
        self
    }

    /// Sets the leading space added per depth level. Resolved against
    /// the theme spacing token. Defaults to
    /// [`Space::sx(2.0)`](crate::Space::sx).
    ///
    /// A node's content sits immediately after its disclosure column;
    /// each deeper level shifts the whole `[indicator][content]` block
    /// right by this amount.
    pub fn indent(mut self, indent: impl Into<ThemeSpace>) -> Self {
        self.indent = indent.into();
        self
    }

    /// Sets the vertical spacing between rows. Defaults to no spacing.
    pub fn spacing(mut self, spacing: impl Into<ThemeSpace>) -> Self {
        self.spacing = spacing.into();
        self
    }

    /// Overrides the disclosure indicator for **both** states with a
    /// single factory closure.
    ///
    /// The closure receives `true` when the node is expanded and
    /// `false` when collapsed, and must return a fresh [`Element`] each
    /// call — every node that has children gets its own indicator, so
    /// the factory is invoked once per visible parent row.
    ///
    /// For the common case of distinct, state-specific widgets, see
    /// [`Tree::expanded_indicator`] and [`Tree::collapsed_indicator`].
    pub fn indicator(
        mut self,
        indicator: impl Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.indicator = Some(Box::new(indicator));
        self
    }

    /// Overrides the indicator shown for an **expanded** node. Defaults
    /// to a downward chevron.
    ///
    /// The factory is invoked once per visible expanded parent row, so
    /// it must return a fresh [`Element`] each call. Setting this
    /// replaces any prior [`Tree::indicator`]/[`Tree::collapsed_indicator`]
    /// configuration, composing them into a single per-state factory.
    pub fn expanded_indicator(
        self,
        indicator: impl Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self
    where
        Message: 'a,
        Theme: iced::widget::text::Catalog + 'a,
        Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    {
        self.indicator_for(true, indicator)
    }

    /// Overrides the indicator shown for a **collapsed** node. Defaults
    /// to a rightward chevron.
    ///
    /// The factory is invoked once per visible collapsed parent row, so
    /// it must return a fresh [`Element`] each call. Setting this
    /// replaces any prior [`Tree::indicator`]/[`Tree::expanded_indicator`]
    /// configuration, composing them into a single per-state factory.
    pub fn collapsed_indicator(
        self,
        indicator: impl Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self
    where
        Message: 'a,
        Theme: iced::widget::text::Catalog + 'a,
        Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    {
        self.indicator_for(false, indicator)
    }

    /// Installs a per-state indicator factory while preserving any
    /// existing factory for the other state.
    fn indicator_for(
        mut self,
        target_expanded: bool,
        indicator: impl Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self
    where
        Message: 'a,
        Theme: iced::widget::text::Catalog + 'a,
        Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    {
        let previous = self.indicator.take();
        self.indicator = Some(Box::new(move |expanded| {
            if expanded == target_expanded {
                indicator()
            } else if let Some(prev) = &previous {
                prev(expanded)
            } else {
                default_indicator(expanded)
            }
        }));
        self
    }
}

impl<'a, Id, Message, Renderer> Tree<'a, Id, Message, crate::Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Sets the style of the [`Tree`] using a
    /// `Fn(&Theme, Status) -> ItemStyle` closure.
    pub fn style(mut self, style: impl Fn(&crate::Theme, Status) -> ItemStyle + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

/// Returns the default indicator element for the given expansion state.
///
/// Uses a Lucide chevron when the `lucide-icons` feature is enabled,
/// otherwise a Unicode triangle.
fn default_indicator<'a, Message, Theme, Renderer>(
    expanded: bool,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    #[cfg(feature = "lucide-icons")]
    {
        use crate::icons::FONT;
        let glyph = if expanded {
            char::from(lucide_icons::Icon::ChevronDown)
        } else {
            char::from(lucide_icons::Icon::ChevronRight)
        };
        text(glyph.to_string()).font(FONT).into()
    }

    #[cfg(not(feature = "lucide-icons"))]
    {
        let glyph = if expanded { '\u{25be}' } else { '\u{25b8}' };
        text(glyph.to_string()).into()
    }
}

impl<'a, Id, Message, Theme, Renderer> Tree<'a, Id, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    Id: Clone,
{
    /// Consumes the tree, flattening the visible nodes (respecting
    /// `expanded`) depth-first into a [`FlatTree`] widget.
    ///
    /// Each visible row contributes exactly two children, in order: its
    /// disclosure indicator (a [`Space`] placeholder for leaf nodes,
    /// which keeps child indexing uniform) and its content element.
    fn flatten(self) -> FlatTree<'a, Id, Message, Theme, Renderer> {
        let Tree {
            roots,
            width,
            height,
            padding,
            node_padding,
            indent,
            spacing,
            on_toggle,
            indicator,
            class,
            ..
        } = self;

        let mut rows = Vec::new();
        let mut children = Vec::new();

        fn walk<'a, Id, Message, Theme, Renderer>(
            nodes: Vec<Node<'a, Id, Message, Theme, Renderer>>,
            depth: usize,
            indicator: &Option<IndicatorFn<'a, Message, Theme, Renderer>>,
            rows: &mut Vec<VisibleRow<Id>>,
            children: &mut Vec<Element<'a, Message, Theme, Renderer>>,
        ) where
            Message: 'a,
            Theme: Catalog + iced::widget::text::Catalog + 'a,
            Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
            Id: Clone,
        {
            for node in nodes {
                let has_children = node.has_children();
                let expanded = node.expanded;

                rows.push(VisibleRow {
                    depth,
                    has_children,
                    id: node.id.clone(),
                });

                // Indicator slot: real element for parents, otherwise a
                // zero-sized placeholder to keep child indexing uniform.
                let indicator_element: Element<'a, Message, Theme, Renderer> = if has_children {
                    match indicator {
                        Some(make) => make(expanded),
                        None => default_indicator(expanded),
                    }
                } else {
                    Space::new().width(0).height(0).into()
                };
                children.push(indicator_element);
                children.push(node.content);

                if expanded && has_children {
                    walk(node.children, depth + 1, indicator, rows, children);
                }
            }
        }

        walk(roots, 0, &indicator, &mut rows, &mut children);

        let config = FlatTreeConfig {
            width,
            height,
            padding,
            node_padding,
            indent,
            spacing,
            on_toggle,
            class,
            _renderer: PhantomData,
        };

        FlatTree {
            rows,
            children,
            config,
        }
    }
}

/// Configuration carried from a [`Tree`] into its flattened
/// [`FlatTree`] widget representation.
struct FlatTreeConfig<'a, Id, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    width: Length,
    height: Length,
    padding: PaddingSource,
    node_padding: PaddingSource,
    indent: ThemeSpace,
    spacing: ThemeSpace,
    on_toggle: Option<ToggleFn<'a, Id, Message>>,
    class: <Theme as Catalog>::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

/// The flattened widget representation of a [`Tree`].
///
/// Holds the visible rows and a flat list of child elements (two per
/// row: indicator then content). All [`Widget`] logic operates over
/// these flat structures, indexing children as `row * 2` (indicator)
/// and `row * 2 + 1` (content).
struct FlatTree<'a, Id, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    rows: Vec<VisibleRow<Id>>,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    config: FlatTreeConfig<'a, Id, Message, Theme, Renderer>,
}

impl<'a, Id, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FlatTree<'a, Id, Message, Theme, Renderer>
where
    Id: Clone,
    Message: Clone,
    Theme: Catalog + SpacingBase,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TreeState>()
    }

    fn state(&self) -> tree::State {
        let initial_padding = self.config.padding.resolve(crate::Theme::DEFAULT_SPACING);
        let initial_indent = self.config.indent.resolve(crate::Theme::DEFAULT_SPACING);
        let initial_spacing = self.config.spacing.resolve(crate::Theme::DEFAULT_SPACING);
        tree::State::new(TreeState {
            rows: vec![RowState::default(); self.rows.len()],
            padding_cache: Cell::new(initial_padding),
            indent_cache: Cell::new(initial_indent),
            spacing_cache: Cell::new(initial_spacing),
        })
    }

    fn children(&self) -> Vec<WidgetTree> {
        self.children.iter().map(WidgetTree::new).collect()
    }

    fn diff(&self, tree: &mut WidgetTree) {
        tree.diff_children(&self.children);

        let state = tree.state.downcast_mut::<TreeState>();
        state.rows.resize_with(self.rows.len(), RowState::default);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.config.width,
            height: self.config.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut WidgetTree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<TreeState>();
        let padding = state.padding_cache.get();
        let indent = state.indent_cache.get();
        let spacing = state.spacing_cache.get();
        let node_padding = self
            .config
            .node_padding
            .resolve(crate::Theme::DEFAULT_SPACING);

        let limits = limits
            .width(self.config.width)
            .height(self.config.height)
            .shrink(padding);

        let available_width = limits
            .resolve(Length::Fill, Length::Shrink, Size::ZERO)
            .width;

        let mut row_nodes = Vec::with_capacity(self.rows.len());
        let mut total_height: f32 = 0.0;

        // Pass 1: lay out every indicator to find the widest disclosure
        // column. Leaves contribute a zero-width placeholder, so only
        // parent rows affect the column width.
        let mut indicator_nodes = Vec::with_capacity(self.rows.len());
        let mut icon_col: f32 = 0.0;
        for i in 0..self.rows.len() {
            let indicator_idx = i * 2;
            let indicator_limits = layout::Limits::new(Size::ZERO, limits.max());
            let indicator_node = self.children[indicator_idx].as_widget_mut().layout(
                &mut tree.children[indicator_idx],
                renderer,
                &indicator_limits,
            );
            icon_col = icon_col.max(indicator_node.size().width);
            indicator_nodes.push(indicator_node);
        }

        // Pass 2: place each row. A node's content sits immediately
        // after its own disclosure column; each depth level adds one
        // `indent` of leading space, so a child's [icon][item] block is
        // shifted right by one indent relative to its parent.
        for (i, row) in self.rows.iter().enumerate() {
            if i > 0 {
                total_height += spacing;
            }

            let content_idx = i * 2 + 1;
            let indicator_node =
                std::mem::replace(&mut indicator_nodes[i], layout::Node::new(Size::ZERO));
            let indicator_size = indicator_node.size();

            let level_inset = node_padding.left + indent * row.depth as f32;
            // Gap between the disclosure column and the content,
            // present only when there is an icon column to separate from.
            let icon_gap = if icon_col > 0.0 { indent / 2.0 } else { 0.0 };
            let content_origin_x = level_inset + icon_col + icon_gap;

            let content_max_width =
                (available_width - content_origin_x - node_padding.right).max(0.0);
            let content_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(content_max_width, limits.max().height),
            );
            let content_node = self.children[content_idx].as_widget_mut().layout(
                &mut tree.children[content_idx],
                renderer,
                &content_limits,
            );
            let content_size = content_node.size();

            let row_inner_height = content_size
                .height
                .max(indicator_size.height)
                .max(node_padding.top + node_padding.bottom);
            let row_height = row_inner_height + node_padding.top + node_padding.bottom;

            // Center the indicator within its fixed-width column, and
            // vertically center both indicator and content in the row.
            let indicator_x =
                (level_inset + (icon_col - indicator_size.width) / 2.0).max(level_inset);
            let indicator_y = (row_height - indicator_size.height) / 2.0;
            let content_y = (row_height - content_size.height) / 2.0;

            let placed_indicator = indicator_node.move_to((indicator_x, indicator_y));
            let placed_content = content_node.move_to((content_origin_x, content_y));

            let row_size = Size::new(available_width, row_height);
            let mut row_node =
                layout::Node::with_children(row_size, vec![placed_indicator, placed_content]);
            row_node = row_node.move_to((padding.left, padding.top + total_height));

            total_height += row_height;
            row_nodes.push(row_node);
        }

        let total_size = Size::new(
            available_width + padding.left + padding.right,
            total_height + padding.top + padding.bottom,
        );

        let resolved = limits.resolve(self.config.width, self.config.height, total_size);
        layout::Node::with_children(resolved.max(total_size), row_nodes)
    }

    fn operate(
        &mut self,
        tree: &mut WidgetTree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for (i, row_layout) in layout.children().enumerate() {
            let mut inner = row_layout.children();
            let indicator_layout = inner.next().unwrap();
            let content_layout = inner.next().unwrap();

            self.children[i * 2].as_widget_mut().operate(
                &mut tree.children[i * 2],
                indicator_layout,
                renderer,
                operation,
            );
            self.children[i * 2 + 1].as_widget_mut().operate(
                &mut tree.children[i * 2 + 1],
                content_layout,
                renderer,
                operation,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut WidgetTree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TreeState>();
        state.rows.resize_with(self.rows.len(), RowState::default);

        // Propagate events to children first (indicator + content).
        for (i, row_layout) in layout.children().enumerate() {
            let mut inner = row_layout.children();
            let indicator_layout = inner.next().unwrap();
            let content_layout = inner.next().unwrap();

            self.children[i * 2].as_widget_mut().update(
                &mut tree.children[i * 2],
                event,
                indicator_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            self.children[i * 2 + 1].as_widget_mut().update(
                &mut tree.children[i * 2 + 1],
                event,
                content_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        // Handle indicator hover/press → toggle.
        for (i, row_layout) in layout.children().enumerate() {
            let row = &self.rows[i];
            if !row.has_children {
                continue;
            }

            let indicator_layout = row_layout.children().next().unwrap();
            let is_over = cursor.is_over(indicator_layout.bounds());

            let row_state = &mut state.rows[i];
            row_state.is_hovered = is_over;

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                    row_state.is_pressed = true;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if row_state.is_pressed
                        && is_over
                        && let Some(on_toggle) = &self.config.on_toggle
                    {
                        shell.publish(on_toggle(row.id.clone()));
                    }
                    row_state.is_pressed = false;
                }
                _ => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &WidgetTree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        // Children first.
        for (i, row_layout) in layout.children().enumerate() {
            let mut inner = row_layout.children();
            let indicator_layout = inner.next().unwrap();
            let content_layout = inner.next().unwrap();

            let indicator_interaction = self.children[i * 2].as_widget().mouse_interaction(
                &tree.children[i * 2],
                indicator_layout,
                cursor,
                viewport,
                renderer,
            );
            if indicator_interaction != mouse::Interaction::None {
                return indicator_interaction;
            }

            let content_interaction = self.children[i * 2 + 1].as_widget().mouse_interaction(
                &tree.children[i * 2 + 1],
                content_layout,
                cursor,
                viewport,
                renderer,
            );
            if content_interaction != mouse::Interaction::None {
                return content_interaction;
            }
        }

        // Pointer over a toggleable indicator.
        if self.config.on_toggle.is_some() {
            for (i, row_layout) in layout.children().enumerate() {
                if self.rows[i].has_children {
                    let indicator_layout = row_layout.children().next().unwrap();
                    if cursor.is_over(indicator_layout.bounds()) {
                        return mouse::Interaction::Pointer;
                    }
                }
            }
        }

        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &WidgetTree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TreeState>();

        // Refresh layout caches from the live theme spacing.
        state
            .padding_cache
            .set(self.config.padding.resolve(theme.spacing()));
        state
            .indent_cache
            .set(self.config.indent.resolve(theme.spacing()));
        state
            .spacing_cache
            .set(self.config.spacing.resolve(theme.spacing()));

        for (i, row_layout) in layout.children().enumerate() {
            let bounds = row_layout.bounds();
            if bounds.intersection(viewport).is_none() {
                continue;
            }

            let row_state = state.rows.get(i).copied().unwrap_or_default();
            let toggleable = self.rows[i].has_children && self.config.on_toggle.is_some();
            let status = if row_state.is_pressed && toggleable {
                Status::Pressed
            } else if row_state.is_hovered && toggleable {
                Status::Hovered
            } else {
                Status::Active
            };

            let style = theme.style(&self.config.class, status);

            if let Some(background) = style.background {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: style.border,
                        ..renderer::Quad::default()
                    },
                    background,
                );
            }

            let mut inner = row_layout.children();
            let indicator_layout = inner.next().unwrap();
            let content_layout = inner.next().unwrap();

            let row_renderer_style = if let Some(color) = style.text_color {
                renderer::Style { text_color: color }
            } else {
                *renderer_style
            };

            self.children[i * 2].as_widget().draw(
                &tree.children[i * 2],
                renderer,
                theme,
                &row_renderer_style,
                indicator_layout,
                cursor,
                viewport,
            );
            self.children[i * 2 + 1].as_widget().draw(
                &tree.children[i * 2 + 1],
                renderer,
                theme,
                &row_renderer_style,
                content_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut WidgetTree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let mut child_trees = tree.children.iter_mut();
        let mut elements = self.children.iter_mut();

        for row_layout in layout.children() {
            let mut inner = row_layout.children();
            let indicator_layout = inner.next().unwrap();
            let content_layout = inner.next().unwrap();

            let indicator_tree = child_trees.next().unwrap();
            let indicator = elements.next().unwrap();
            if let Some(overlay) = indicator.as_widget_mut().overlay(
                indicator_tree,
                indicator_layout,
                renderer,
                viewport,
                translation,
            ) {
                return Some(overlay);
            }

            let content_tree = child_trees.next().unwrap();
            let content = elements.next().unwrap();
            if let Some(overlay) = content.as_widget_mut().overlay(
                content_tree,
                content_layout,
                renderer,
                viewport,
                translation,
            ) {
                return Some(overlay);
            }
        }
        None
    }
}

impl<'a, Id, Message, Theme, Renderer> From<Tree<'a, Id, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(tree: Tree<'a, Id, Message, Theme, Renderer>) -> Self {
        Self::new(tree.flatten())
    }
}
