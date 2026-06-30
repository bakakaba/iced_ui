//! Horizontal group of toggle segments where exactly one (or more) can be selected.
//!
//! # Example
//!
//! ```ignore
//! use iced_ui::segmented_button::{Segment, SegmentedButton};
//!
//! let button = SegmentedButton::new()
//!     .push(Segment::new(text("Day")), selected == 0)
//!     .push(Segment::new(text("Week")), selected == 1)
//!     .push(Segment::new(text("Month")), selected == 2)
//!     .on_press(Message::TabChanged);
//! ```

mod style;

pub use style::{Catalog, SegmentStatus, SegmentStyle, StyleFn, default};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Element, Event, Length, Rectangle, Size};

use crate::{Roundness, RoundnessBase, SpacingBase};

/// Height of the segmented button in logical pixels (MD3 spec).
const SEGMENT_HEIGHT: f32 = 40.0;

/// A single segment within a [`SegmentedButton`].
///
/// Each segment has a required label and an optional leading icon.
pub struct Segment<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    label: Element<'a, Message, Theme, Renderer>,
    icon: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Segment<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new segment with the given label content.
    pub fn new(label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    /// Sets a leading icon element for this segment.
    pub fn icon(mut self, icon: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    fn child_count(&self) -> usize {
        1 + self.icon.is_some() as usize
    }
}

/// Internal per-segment interaction state.
#[derive(Debug, Clone, Default)]
struct SegmentState {
    is_hovered: bool,
    is_pressed: bool,
}

/// Internal state for the whole segmented button group.
#[derive(Debug, Clone, Default)]
struct State {
    segments: Vec<SegmentState>,
}

/// An entry held by the [`SegmentedButton`]: a segment plus its
/// selected flag.
struct Entry<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    segment: Segment<'a, Message, Theme, Renderer>,
    selected: bool,
}

/// Draws as a horizontal row of [`Segment`]s. The first segment gets
/// left-rounded corners, the last gets right-rounded corners, and
/// middle segments have square corners. A 1px border outlines the
/// entire group.
pub struct SegmentedButton<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    entries: Vec<Entry<'a, Message, Theme, Renderer>>,
    on_press: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    roundness: Option<Roundness>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Default for SegmentedButton<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> SegmentedButton<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Creates a new empty segmented button group.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            on_press: None,
            roundness: None,
            class: Theme::default(),
        }
    }

    /// Appends a segment with its selected state.
    pub fn push(mut self, segment: Segment<'a, Message, Theme, Renderer>, selected: bool) -> Self {
        self.entries.push(Entry { segment, selected });
        self
    }

    /// Sets the callback invoked when a segment is pressed. The
    /// callback receives the zero-based index of the pressed segment.
    pub fn on_press(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(f));
        self
    }

    /// Sets the style class.
    pub fn style(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Overrides the corner roundness of the group's outer corners,
    /// bypassing the theme's default for this widget. Accepts a
    /// [`Roundness`] token: [`Roundness::sx`] scales the theme's
    /// roundness base, [`Roundness::px`] sets an absolute radius.
    pub fn roundness(mut self, roundness: Roundness) -> Self {
        self.roundness = Some(roundness);
        self
    }

    /// Total number of child elements across all segments.
    fn total_children(&self) -> usize {
        self.entries.iter().map(|e| e.segment.child_count()).sum()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SegmentedButton<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            segments: vec![SegmentState::default(); self.entries.len()],
        })
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(self.total_children());
        for entry in &self.entries {
            if let Some(icon) = &entry.segment.icon {
                children.push(Tree::new(icon));
            }
            children.push(Tree::new(&entry.segment.label));
        }
        children
    }

    fn diff(&self, tree: &mut Tree) {
        // Ensure per-segment state vector stays in sync.
        let state = tree.state.downcast_mut::<State>();
        state
            .segments
            .resize_with(self.entries.len(), Default::default);

        let mut refs: Vec<&Element<'_, Message, Theme, Renderer>> =
            Vec::with_capacity(self.total_children());
        for entry in &self.entries {
            if let Some(icon) = &entry.segment.icon {
                refs.push(icon);
            }
            refs.push(&entry.segment.label);
        }
        tree.diff_children(&refs);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Fixed(SEGMENT_HEIGHT))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let h_padding = 12.0_f32;
        let icon_gap = 8.0_f32;
        let icon_max = 18.0_f32;

        let max_width = limits.max().width;
        let segment_count = self.entries.len();
        if segment_count == 0 {
            return layout::Node::new(Size::new(0.0, SEGMENT_HEIGHT));
        }

        // Layout each segment and collect nodes per segment.
        let mut segment_nodes: Vec<Vec<layout::Node>> = Vec::with_capacity(segment_count);
        let mut segment_widths: Vec<f32> = Vec::with_capacity(segment_count);
        let mut child_idx = 0;

        for entry in &mut self.entries {
            let mut nodes = Vec::new();
            let mut x_cursor = h_padding;

            // Icon
            if let Some(icon) = &mut entry.segment.icon {
                let icon_limits = layout::Limits::new(Size::ZERO, Size::new(icon_max, icon_max));
                let mut node = icon.as_widget_mut().layout(
                    &mut tree.children[child_idx],
                    renderer,
                    &icon_limits,
                );
                let icon_y = (SEGMENT_HEIGHT - node.size().height) / 2.0;
                node = node.move_to(iced::Point::new(x_cursor, icon_y));
                x_cursor += node.size().width + icon_gap;
                nodes.push(node);
                child_idx += 1;
            }

            // Label
            let label_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(
                    (max_width / segment_count as f32) - h_padding * 2.0,
                    SEGMENT_HEIGHT,
                ),
            );
            let mut label_node = entry.segment.label.as_widget_mut().layout(
                &mut tree.children[child_idx],
                renderer,
                &label_limits,
            );
            let label_y = (SEGMENT_HEIGHT - label_node.size().height) / 2.0;
            label_node = label_node.move_to(iced::Point::new(x_cursor, label_y));
            x_cursor += label_node.size().width;
            nodes.push(label_node);
            child_idx += 1;

            x_cursor += h_padding;
            segment_widths.push(x_cursor);
            segment_nodes.push(nodes);
        }

        // Build top-level children: one node per segment wrapping its
        // icon/label children.
        let mut top_children = Vec::with_capacity(segment_count);
        let mut total_x = 0.0_f32;

        for (nodes, &width) in segment_nodes.into_iter().zip(segment_widths.iter()) {
            let seg_size = Size::new(width, SEGMENT_HEIGHT);
            let mut seg_node = layout::Node::with_children(seg_size, nodes);
            seg_node = seg_node.move_to(iced::Point::new(total_x, 0.0));
            top_children.push(seg_node);
            total_x += width;
        }

        layout::Node::with_children(Size::new(total_x, SEGMENT_HEIGHT), top_children)
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
        let seg_count = self.entries.len();

        let mut child_idx = 0;

        for (i, (entry, seg_layout)) in self.entries.iter().zip(layout.children()).enumerate() {
            let seg_state = state.segments.get(i).cloned().unwrap_or_default();
            let bounds = seg_layout.bounds();

            let status = if seg_state.is_pressed {
                SegmentStatus::Pressed
            } else if seg_state.is_hovered {
                SegmentStatus::Hovered
            } else {
                SegmentStatus::Active
            };

            let seg_style = Catalog::style(theme, &self.class, entry.selected, status);

            // Use the radius from the style, then zero-out corners based
            // on position within the group. A `.roundness(..)` override
            // replaces the uniform radius.
            let r = seg_style.border.radius;
            let base_r = match self.roundness {
                Some(roundness) => roundness.resolve(RoundnessBase::roundness(theme)),
                None => r.top_left, // uniform radius from style fn
            };

            let border = if seg_count == 1 {
                Border {
                    radius: base_r.into(),
                    ..seg_style.border
                }
            } else if i == 0 {
                // First: rounded left, square right.
                Border {
                    radius: iced::border::Radius {
                        top_left: base_r,
                        top_right: 0.0,
                        bottom_right: 0.0,
                        bottom_left: base_r,
                    },
                    ..seg_style.border
                }
            } else if i == seg_count - 1 {
                // Last: square left, rounded right.
                Border {
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: base_r,
                        bottom_right: base_r,
                        bottom_left: 0.0,
                    },
                    ..seg_style.border
                }
            } else {
                // Middle: no rounding.
                Border {
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                    },
                    ..seg_style.border
                }
            };

            // Draw segment background.
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border,
                    ..renderer::Quad::default()
                },
                seg_style
                    .background
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );

            // Draw child elements (icon + label).
            let text_style = renderer::Style {
                text_color: seg_style.text_color,
            };

            let seg_child_start = child_idx;
            for child_layout in seg_layout.children() {
                if child_idx < tree.children.len() {
                    let local_idx = child_idx - seg_child_start;
                    let child_widget: &dyn Widget<Message, Theme, Renderer> =
                        if let Some(icon) = &entry.segment.icon {
                            match local_idx {
                                0 => icon.as_widget(),
                                _ => entry.segment.label.as_widget(),
                            }
                        } else {
                            entry.segment.label.as_widget()
                        };

                    child_widget.draw(
                        &tree.children[child_idx],
                        renderer,
                        theme,
                        &text_style,
                        child_layout,
                        cursor,
                        viewport,
                    );
                }
                child_idx += 1;
            }
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
        // Ensure state vector length matches entries.
        state
            .segments
            .resize_with(self.entries.len(), Default::default);

        // Forward events to children.
        let mut child_idx = 0;
        for (seg_layout, entry) in layout.children().zip(self.entries.iter_mut()) {
            let seg_child_start = child_idx;
            for child_layout in seg_layout.children() {
                if child_idx < tree.children.len() {
                    let local_idx = child_idx - seg_child_start;
                    let widget: &mut dyn Widget<Message, Theme, Renderer> =
                        if let Some(icon) = &mut entry.segment.icon {
                            match local_idx {
                                0 => icon.as_widget_mut(),
                                _ => entry.segment.label.as_widget_mut(),
                            }
                        } else {
                            entry.segment.label.as_widget_mut()
                        };

                    widget.update(
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

        // Handle segment-level interactions.
        for (i, seg_layout) in layout.children().enumerate() {
            let bounds = seg_layout.bounds();
            let is_over = cursor.is_over(bounds);

            if let Some(seg_state) = state.segments.get_mut(i) {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                        seg_state.is_hovered = is_over;
                        if !is_over {
                            seg_state.is_pressed = false;
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if is_over => {
                        seg_state.is_pressed = true;
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        if seg_state.is_pressed
                            && is_over
                            && let Some(on_press) = &self.on_press
                        {
                            shell.publish(on_press(i));
                        }
                        seg_state.is_pressed = false;
                    }
                    _ => {}
                }
            }
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
        if self.on_press.is_some() {
            for seg_layout in layout.children() {
                if cursor.is_over(seg_layout.bounds()) {
                    return mouse::Interaction::Pointer;
                }
            }
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
        for (seg_layout, entry) in layout.children().zip(self.entries.iter_mut()) {
            let seg_child_start = child_idx;
            for child_layout in seg_layout.children() {
                if child_idx < tree.children.len() {
                    let local_idx = child_idx - seg_child_start;
                    let widget: &mut dyn Widget<Message, Theme, Renderer> =
                        if let Some(icon) = &mut entry.segment.icon {
                            match local_idx {
                                0 => icon.as_widget_mut(),
                                _ => entry.segment.label.as_widget_mut(),
                            }
                        } else {
                            entry.segment.label.as_widget_mut()
                        };

                    widget.operate(
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
}

impl<'a, Message, Theme, Renderer> From<SegmentedButton<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + SpacingBase + RoundnessBase + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(button: SegmentedButton<'a, Message, Theme, Renderer>) -> Self {
        Element::new(button)
    }
}
