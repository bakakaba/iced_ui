//! The floating dropdown / submenu [`Overlay`] used by
//! [`MenuBar`](super::MenuBar).

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay::Overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph, Text};
use iced::advanced::{Clipboard, Shell};
use iced::alignment;
use iced::mouse;
use iced::{Border, Color, Event, Font, Padding, Pixels, Point, Rectangle, Size, keyboard};

use super::State;
use super::item::{Entry, Menu};
use super::style::{Catalog, Style};

/// Layout metrics derived from the active [`Style`] and widget
/// configuration.
#[derive(Debug, Clone, Copy)]
pub(super) struct Metrics {
    pub text_size: f32,
    pub shortcut_text_size: f32,
    pub row_height: f32,
    /// Padding around the **bar** of top-level labels. Not used for
    /// dropdown popups — those use [`popup_padding`] instead.
    pub padding: Padding,
    /// Padding applied inside dropdown popups. Top/bottom match
    /// [`padding`]; left/right come from `Style::spacing` so that the
    /// popup's interior breathing room tracks the theme's general
    /// spacing value.
    pub popup_padding: Padding,
    pub icon_cell: f32,
    pub gap: f32,
    pub separator_height: f32,
    pub submenu_indicator_width: f32,
    pub min_width: f32,
}

impl Metrics {
    pub fn new(text_size: f32, padding: Padding, spacing: f32) -> Self {
        Self {
            text_size,
            shortcut_text_size: (text_size - 2.0).max(8.0),
            row_height: (text_size * 1.3).ceil() + padding.top + padding.bottom,
            padding,
            popup_padding: Padding {
                top: padding.top,
                bottom: padding.bottom,
                left: spacing,
                right: spacing,
            },
            icon_cell: text_size,
            gap: spacing,
            separator_height: 9.0,
            submenu_indicator_width: text_size * 0.75,
            min_width: 160.0,
        }
    }
}

/// The [`Overlay`] rendered whenever a top-level menu is open.
pub(super) struct MenuOverlay<'a, 'b, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    pub(super) menus: &'b mut [Menu<Message>],
    pub(super) state: &'b mut State<Renderer::Paragraph>,
    pub(super) bar_bounds: Rectangle,
    pub(super) bar_label_bounds: Vec<Rectangle>,
    pub(super) metrics: Metrics,
    pub(super) font: Font,
    pub(super) style_fn: &'b <Theme as Catalog>::Class<'a>,
}

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for MenuOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font>,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let Some(path) = self.state.open_path.clone() else {
            return layout::Node::new(Size::ZERO);
        };

        let mut children: Vec<layout::Node> = Vec::with_capacity(path.len());

        // Start with the top-level menu.
        let mut current_menu: &Menu<Message> = &self.menus[path[0]];

        // Anchor the top-level dropdown so that the first glyph of each
        // dropdown item's label text lines up under the first glyph of
        // the bar label. `label_offset` is the horizontal distance from
        // the popup's left edge to where `draw_menu` renders the label
        // text (see `label_x` in `draw_menu`).
        let top_label = self.bar_label_bounds.get(path[0]);
        let label_offset = {
            let has_icon = current_menu
                .entries
                .iter()
                .any(|e| matches!(e, Entry::Item(i) if i.icon.is_some()));
            let icon_cell = if has_icon {
                self.metrics.icon_cell
            } else {
                0.0
            };
            self.metrics.popup_padding.left + icon_cell
        };
        let mut anchor_origin = Point::new(
            top_label
                .map(|r| (r.x + self.metrics.padding.left - label_offset).max(0.0))
                .unwrap_or(self.bar_bounds.x),
            top_label
                .map(|r| r.y + r.height)
                .unwrap_or(self.bar_bounds.y + self.bar_bounds.height),
        );

        for (depth, _) in path.iter().enumerate() {
            let node = layout_menu::<Message, Renderer>(
                current_menu,
                renderer,
                self.font,
                self.metrics,
                anchor_origin,
                bounds,
            );

            let next_idx = path.get(depth + 1).copied();

            if let Some(next) = next_idx {
                // Row bounds inside the just-laid-out node.
                let row_top = node.bounds().y
                    + self.metrics.popup_padding.top
                    + row_offset(current_menu, next, self.metrics);
                anchor_origin = Point::new(node.bounds().x + node.bounds().width, row_top);
            }

            children.push(node);

            if let Some(next) = next_idx {
                if let Some(Entry::Submenu(sub)) = current_menu.entries.get(next) {
                    current_menu = sub;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let total_bounds = children
            .iter()
            .map(layout::Node::bounds)
            .fold::<Option<Rectangle>, _>(None, |acc, b| match acc {
                Some(a) => Some(a.union(&b)),
                None => Some(b),
            })
            .unwrap_or(Rectangle::with_size(Size::ZERO));

        // Children were laid out in absolute coordinates via
        // `.move_to(anchor_origin)` inside `layout_menu`. iced's layout
        // traversal will add the outer node's position to each child's
        // own position, so we rewrite child origins relative to the
        // outer node before returning.
        let children: Vec<layout::Node> = children
            .into_iter()
            .map(|node| {
                let b = node.bounds();
                node.move_to(Point::new(b.x - total_bounds.x, b.y - total_bounds.y))
            })
            .collect();

        layout::Node::with_children(total_bounds.size(), children)
            .move_to(Point::new(total_bounds.x, total_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let Some(path) = self.state.open_path.as_ref() else {
            return;
        };
        let style = theme.style(self.style_fn);

        let mut current_menu: &Menu<Message> = &self.menus[path[0]];
        for (depth, child_layout) in layout.children().enumerate() {
            let hovered = self.state.hover_path.get(depth).copied().flatten();
            let opened = path.get(depth + 1).copied();

            draw_menu::<Message, Renderer>(
                renderer,
                current_menu,
                child_layout,
                cursor,
                &style,
                self.metrics,
                self.font,
                hovered,
                opened,
            );

            if let Some(next) = opened {
                if let Some(Entry::Submenu(sub)) = current_menu.entries.get(next) {
                    current_menu = sub;
                } else {
                    break;
                }
            }
        }
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(path) = self.state.open_path.clone() else {
            return;
        };

        let child_layouts: Vec<Layout<'_>> = layout.children().collect();
        let child_bounds: Vec<Rectangle> = child_layouts.iter().map(|l| l.bounds()).collect();

        // Compute invisible hover-bridge rectangles between the bar and
        // top-level panel, and between adjacent open panels. These keep
        // the path stable while the cursor briefly transits the seam
        // between two open panels (or the gap between the bar and the
        // dropdown).
        let bridges = hover_bridges(
            path.len(),
            &child_bounds,
            &self.bar_label_bounds,
            path[0],
            self.metrics,
        );

        // Recompute hovered row per depth based on current cursor.
        // If the cursor is over a panel, hit-test it. Otherwise, if the
        // cursor is over the bridge associated with that depth, inherit
        // the previously hovered row at that depth so the highlighted
        // row remains stable during transit.
        let mut hovered: Vec<Option<usize>> = Vec::with_capacity(path.len());
        let mut current_menu: &Menu<Message> = &self.menus[path[0]];
        for (depth, _) in path.iter().enumerate() {
            let Some(child_layout) = child_layouts.get(depth) else {
                hovered.push(None);
                break;
            };
            let row = match cursor.position() {
                Some(p) if child_layout.bounds().contains(p) => {
                    hit_row(current_menu, *child_layout, self.metrics, p)
                }
                Some(p)
                    if bridges
                        .get(depth)
                        .and_then(|b| *b)
                        .is_some_and(|r| r.contains(p)) =>
                {
                    // In the bridge for this depth — preserve the prior
                    // hovered row so highlighting doesn't flicker.
                    self.state.hover_path.get(depth).copied().flatten()
                }
                _ => None,
            };
            hovered.push(row);

            if let Some(next) = path.get(depth + 1).copied() {
                if let Some(Entry::Submenu(sub)) = current_menu.entries.get(next) {
                    current_menu = sub;
                } else {
                    break;
                }
            }
        }

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_pos) = cursor.position() {
                    match compute_hover_update(
                        self.menus,
                        &path,
                        &self.state.hover_path,
                        &child_bounds,
                        &self.bar_label_bounds,
                        &bridges,
                        self.metrics,
                        cursor_pos,
                    ) {
                        HoverUpdate::SwitchTopLevel(i) => {
                            self.state.open_path = Some(vec![i]);
                            self.state.bar_active = Some(i);
                            self.state.hover_path = vec![Some(i)];
                            shell.request_redraw();
                        }
                        HoverUpdate::Rebuild {
                            open_path,
                            hover_path,
                        } => {
                            self.state.open_path = Some(open_path);
                            if self.state.hover_path != hover_path {
                                self.state.hover_path = hover_path;
                            }
                            shell.request_redraw();
                        }
                        HoverUpdate::Preserve { hover_path } => {
                            if self.state.hover_path != hover_path {
                                self.state.hover_path = hover_path;
                                shell.request_redraw();
                            }
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(cursor_pos) = cursor.position() else {
                    return;
                };

                let mut current_menu: &Menu<Message> = &self.menus[path[0]];
                for (depth, child_layout) in child_layouts.iter().enumerate() {
                    if let Some(row_idx) =
                        hit_row(current_menu, *child_layout, self.metrics, cursor_pos)
                    {
                        match current_menu.entries.get(row_idx) {
                            Some(Entry::Item(item)) if item.is_activatable() => {
                                if let Some(msg) = item.on_press.clone() {
                                    shell.publish(msg);
                                }
                                self.state.close();
                                self.state.bar_active = None;
                                shell.capture_event();
                                shell.request_redraw();
                                return;
                            }
                            Some(Entry::Submenu(_)) => {
                                let mut new_path =
                                    path.iter().take(depth + 1).copied().collect::<Vec<_>>();
                                new_path.push(row_idx);
                                self.state.open_path = Some(new_path);
                                shell.capture_event();
                                shell.request_redraw();
                                return;
                            }
                            _ => {}
                        }
                    }

                    if let Some(next) = path.get(depth + 1).copied() {
                        if let Some(Entry::Submenu(sub)) = current_menu.entries.get(next) {
                            current_menu = sub;
                        } else {
                            break;
                        }
                    }
                }

                // Click outside any popup and outside the bar: close.
                let in_popup = child_layouts
                    .iter()
                    .any(|l| l.bounds().contains(cursor_pos));
                if !in_popup && !self.bar_bounds.contains(cursor_pos) {
                    self.state.close();
                    self.state.bar_active = None;
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key {
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    if self.state.open_path.as_ref().is_some_and(|p| p.len() > 1) {
                        if let Some(p) = self.state.open_path.as_mut() {
                            p.pop();
                        }
                    } else {
                        self.state.close();
                        self.state.bar_active = None;
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                    if self.state.open_path.as_ref().is_some_and(|p| p.len() > 1) {
                        if let Some(p) = self.state.open_path.as_mut() {
                            p.pop();
                        }
                    } else if !self.bar_label_bounds.is_empty() {
                        let n = self.bar_label_bounds.len();
                        let cur = path[0];
                        let prev = (cur + n - 1) % n;
                        self.state.open_path = Some(vec![prev]);
                        self.state.bar_active = Some(prev);
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                    let tail_menu = walk_menu(self.menus, &path);
                    if let Some(row) = hovered.last().copied().flatten()
                        && let Some(Entry::Submenu(_)) = tail_menu.entries.get(row)
                    {
                        let mut new_path = path.clone();
                        new_path.push(row);
                        self.state.open_path = Some(new_path);
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                    if !self.bar_label_bounds.is_empty() {
                        let n = self.bar_label_bounds.len();
                        let cur = path[0];
                        let next = (cur + 1) % n;
                        self.state.open_path = Some(vec![next]);
                        self.state.bar_active = Some(next);
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => {
                    let menu = walk_menu(self.menus, &path);
                    if let Some(row) = hovered.last().copied().flatten() {
                        match menu.entries.get(row) {
                            Some(Entry::Item(item)) if item.is_activatable() => {
                                if let Some(msg) = item.on_press.clone() {
                                    shell.publish(msg);
                                }
                                self.state.close();
                                self.state.bar_active = None;
                                shell.capture_event();
                                shell.request_redraw();
                            }
                            Some(Entry::Submenu(_)) => {
                                let mut new_path = path.clone();
                                new_path.push(row);
                                self.state.open_path = Some(new_path);
                                shell.capture_event();
                                shell.request_redraw();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(pos) = cursor.position() else {
            return mouse::Interaction::None;
        };
        for child in layout.children() {
            if child.bounds().contains(pos) {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::None
    }

    fn index(&self) -> f32 {
        1.0
    }
}

fn row_offset<Message>(menu: &Menu<Message>, index: usize, metrics: Metrics) -> f32 {
    let mut y = 0.0;
    for (i, entry) in menu.entries.iter().enumerate() {
        if i == index {
            break;
        }
        y += match entry {
            Entry::Separator => metrics.separator_height,
            _ => metrics.row_height,
        };
    }
    y
}

fn hit_row<Message>(
    menu: &Menu<Message>,
    layout: Layout<'_>,
    metrics: Metrics,
    cursor: Point,
) -> Option<usize> {
    let bounds = layout.bounds();
    if !bounds.contains(cursor) {
        return None;
    }
    let mut y = bounds.y + metrics.padding.top;
    for (i, entry) in menu.entries.iter().enumerate() {
        let h = match entry {
            Entry::Separator => metrics.separator_height,
            _ => metrics.row_height,
        };
        if cursor.y >= y && cursor.y < y + h {
            return match entry {
                Entry::Separator => None,
                _ => Some(i),
            };
        }
        y += h;
    }
    None
}

/// Compute invisible "hover bridge" rectangles for each open panel.
///
/// Bridges are hit-test only geometry that buffer the gap between
/// adjacent panels (and between the bar and the top-level dropdown) so
/// that brief excursions of the cursor across panel edges do not cause
/// the path-rebuild logic to truncate `open_path` while the user is
/// still on their way to a deeper panel. Bridge thickness equals
/// `metrics.gap` (i.e. the theme's `Style::spacing`).
///
/// Returns one `Option<Rectangle>` per depth in `path`. `None` indicates
/// no bridge is needed at that depth.
fn hover_bridges(
    path_len: usize,
    child_layouts: &[Rectangle],
    bar_label_bounds: &[Rectangle],
    top_label_idx: usize,
    metrics: Metrics,
) -> Vec<Option<Rectangle>> {
    let mut bridges: Vec<Option<Rectangle>> = Vec::with_capacity(path_len);
    let gap = metrics.gap.max(0.0);

    // Depth 0: bridge between the bar label and the top-level dropdown.
    if let (Some(panel), Some(label)) = (
        child_layouts.first().copied(),
        bar_label_bounds.get(top_label_idx).copied(),
    ) {
        // Vertical span between the bar label's bottom edge and the
        // panel's top edge. If the panel is above the label (bar
        // anchored at the bottom of the screen, etc.), mirror.
        let label_bottom = label.y + label.height;
        let panel_top = panel.y;
        let panel_bottom = panel.y + panel.height;

        let v_rect = if panel_top >= label_bottom {
            // Standard: panel below bar.
            let mut top = label_bottom - gap;
            let bottom = panel_top + gap;
            if top > bottom {
                top = bottom;
            }
            Some((top, bottom))
        } else if panel_bottom <= label.y {
            // Flipped: panel above bar.
            let top = panel_bottom - gap;
            let mut bottom = label.y + gap;
            if bottom < top {
                bottom = top;
            }
            Some((top, bottom))
        } else {
            None
        };

        bridges.push(v_rect.map(|(top, bottom)| {
            // x range: intersection of label's x-range and panel's x-range.
            let x0 = label.x.max(panel.x);
            let x1 = (label.x + label.width).min(panel.x + panel.width);
            if x1 > x0 {
                Rectangle {
                    x: x0,
                    y: top,
                    width: x1 - x0,
                    height: (bottom - top).max(0.0),
                }
            } else {
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                }
            }
        }));
    } else {
        bridges.push(None);
    }

    // Depth >= 1: bridge on the parent-facing edge of the child panel,
    // plus a symmetric strip on the parent's side, forming a hover
    // tunnel of total width 2 * gap centered on the shared edge.
    for depth in 1..path_len {
        let (Some(parent), Some(child)) = (
            child_layouts.get(depth - 1).copied(),
            child_layouts.get(depth).copied(),
        ) else {
            bridges.push(None);
            continue;
        };

        let parent_right = parent.x + parent.width;
        let parent_left = parent.x;
        let child_right = child.x + child.width;
        let child_left = child.x;

        // Detect side: child to the right of parent (standard) vs.
        // child to the left of parent (flipped due to viewport overflow).
        let (x0, x1) = if child_left >= parent_right - 0.5 {
            // Standard: child on right.
            (parent_right - gap, child_left + gap)
        } else if child_right <= parent_left + 0.5 {
            // Flipped: child on left.
            (child_right - gap, parent_left + gap)
        } else {
            // Overlapping panels — no bridge needed.
            bridges.push(None);
            continue;
        };

        // Vertical extent: union of the parent and child y-ranges so
        // the bridge spans both panels' vertical reach.
        let y0 = parent.y.min(child.y);
        let y1 = (parent.y + parent.height).max(child.y + child.height);

        if x1 > x0 && y1 > y0 {
            bridges.push(Some(Rectangle {
                x: x0,
                y: y0,
                width: x1 - x0,
                height: y1 - y0,
            }));
        } else {
            bridges.push(None);
        }
    }

    bridges
}

/// Outcome of evaluating a `CursorMoved` event against the menu
/// overlay's current state. Pure data so the decision logic can be
/// unit-tested in isolation from iced's event plumbing.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum HoverUpdate {
    /// The cursor is over a different top-level bar label than the one
    /// currently open. Switch to it.
    SwitchTopLevel(usize),
    /// The open path and/or hovered rows changed.
    Rebuild {
        open_path: Vec<usize>,
        hover_path: Vec<Option<usize>>,
    },
    /// The open path is unchanged; only the hover highlights may differ.
    Preserve { hover_path: Vec<Option<usize>> },
}

/// Pure helper that decides what should happen on a `CursorMoved`
/// event.
///
/// Splitting this out from [`MenuOverlay::update`] makes the hover
/// behaviour testable without spinning up a real iced runtime: callers
/// in tests construct synthetic [`Rectangle`] values for `child_bounds`
/// and `bar_label_bounds`, build a small [`Menu`] tree, and feed a
/// cursor [`Point`] through this function.
#[allow(clippy::too_many_arguments)]
pub(super) fn compute_hover_update<Message>(
    menus: &[Menu<Message>],
    path: &[usize],
    prev_hover_path: &[Option<usize>],
    child_bounds: &[Rectangle],
    bar_label_bounds: &[Rectangle],
    bridges: &[Option<Rectangle>],
    metrics: Metrics,
    cursor: Point,
) -> HoverUpdate {
    debug_assert!(!path.is_empty());

    // Bar-label switch.
    for (i, label_bounds) in bar_label_bounds.iter().enumerate() {
        if label_bounds.contains(cursor) && path[0] != i {
            return HoverUpdate::SwitchTopLevel(i);
        }
    }

    // Compute hovered row per depth.
    let mut hovered: Vec<Option<usize>> = Vec::with_capacity(path.len());
    let mut current_menu: &Menu<Message> = &menus[path[0]];
    for (depth, _) in path.iter().enumerate() {
        let Some(panel) = child_bounds.get(depth).copied() else {
            hovered.push(None);
            break;
        };
        let row = if panel.contains(cursor) {
            hit_row_in_bounds(current_menu, panel, metrics, cursor)
        } else if bridges
            .get(depth)
            .and_then(|b| *b)
            .is_some_and(|r| r.contains(cursor))
        {
            // Inside the bridge — preserve the previously highlighted row
            // so the highlight does not flicker during transit.
            prev_hover_path.get(depth).copied().flatten()
        } else {
            None
        };
        hovered.push(row);

        if let Some(next) = path.get(depth + 1).copied() {
            if let Some(Entry::Submenu(sub)) = current_menu.entries.get(next) {
                current_menu = sub;
            } else {
                break;
            }
        }
    }

    // Bridge guard: if cursor is outside every panel but inside a
    // bridge, preserve the current path entirely (this is the
    // pre-existing transit-forgiveness behaviour).
    let in_any_panel = child_bounds.iter().any(|p| p.contains(cursor));
    let in_any_bridge = bridges
        .iter()
        .filter_map(|b| *b)
        .any(|r| r.contains(cursor));
    if !in_any_panel && in_any_bridge {
        return HoverUpdate::Preserve {
            hover_path: hovered,
        };
    }

    // Find the deepest depth whose panel actually contains the cursor.
    // This is the "anchor" — panels at or above this depth must remain
    // open. Walking the rebuild from depth 0 (as the previous
    // implementation did) would close deeper panels the moment the
    // cursor crossed an ancestor's boundary, which is precisely the
    // submenu-inaccessible bug.
    let anchor = (0..path.len())
        .rev()
        .find(|&d| child_bounds.get(d).is_some_and(|p| p.contains(cursor)));

    let new_path: Vec<usize> = match anchor {
        None => {
            // Cursor outside every open panel and outside every
            // bridge. Preserve the path — closing on outside hover is
            // not desired (matches the bar's `CursorLeft` policy).
            path.to_vec()
        }
        Some(d) => {
            let menu_at_d = walk_menu(menus, &path[..=d]);
            let panel = child_bounds[d];
            let row = hit_row_in_bounds(menu_at_d, panel, metrics, cursor);

            match row {
                None => {
                    // Cursor is inside the panel but on padding or a
                    // separator. Preserve the entire path so deeper
                    // panels do not flicker closed.
                    path.to_vec()
                }
                Some(r) => {
                    let mut np: Vec<usize> = path.iter().take(d + 1).copied().collect();
                    if let Some(Entry::Submenu(_)) = menu_at_d.entries.get(r) {
                        np.push(r);
                        // Auto-extend deeper if subsequent depths in
                        // `hovered` continue to point to submenu rows.
                        // Preserves the current behaviour where
                        // hovering a submenu row in the active branch
                        // automatically opens its child.
                        let mut walk = match menu_at_d.entries.get(r) {
                            Some(Entry::Submenu(sub)) => sub,
                            _ => unreachable!(),
                        };
                        for sub_row in hovered.iter().skip(d + 1) {
                            let Some(idx) = *sub_row else { break };
                            match walk.entries.get(idx) {
                                Some(Entry::Submenu(deeper)) => {
                                    np.push(idx);
                                    walk = deeper;
                                }
                                _ => break,
                            }
                        }
                    }
                    // For non-submenu rows the path is truncated to
                    // [..=d], legitimately closing any deeper panels.
                    np
                }
            }
        }
    };

    if new_path != path {
        HoverUpdate::Rebuild {
            open_path: new_path,
            hover_path: hovered,
        }
    } else {
        HoverUpdate::Preserve {
            hover_path: hovered,
        }
    }
}

/// Variant of [`hit_row`] that operates on a raw [`Rectangle`] so it
/// can be called from [`compute_hover_update`] without a [`Layout`].
fn hit_row_in_bounds<Message>(
    menu: &Menu<Message>,
    bounds: Rectangle,
    metrics: Metrics,
    cursor: Point,
) -> Option<usize> {
    if !bounds.contains(cursor) {
        return None;
    }
    let mut y = bounds.y + metrics.padding.top;
    for (i, entry) in menu.entries.iter().enumerate() {
        let h = match entry {
            Entry::Separator => metrics.separator_height,
            _ => metrics.row_height,
        };
        if cursor.y >= y && cursor.y < y + h {
            return match entry {
                Entry::Separator => None,
                _ => Some(i),
            };
        }
        y += h;
    }
    None
}

fn walk_menu<'m, Message>(menus: &'m [Menu<Message>], path: &[usize]) -> &'m Menu<Message> {
    let mut cur = &menus[path[0]];
    for &i in &path[1..] {
        if let Some(Entry::Submenu(sub)) = cur.entries.get(i) {
            cur = sub;
        } else {
            break;
        }
    }
    cur
}

fn layout_menu<Message, Renderer>(
    menu: &Menu<Message>,
    renderer: &Renderer,
    font: Font,
    metrics: Metrics,
    anchor_origin: Point,
    viewport: Size,
) -> layout::Node
where
    Renderer: text::Renderer<Font = Font>,
{
    let mut content_width: f32 = 0.0;
    let mut has_icon = false;

    for entry in &menu.entries {
        match entry {
            Entry::Item(item) => {
                let icon_font = item.icon.as_ref().and_then(|i| i.font).unwrap_or(font);
                let label_width =
                    measure::<Renderer>(renderer, &item.label, metrics.text_size, font);
                let mut row_w = label_width;
                if let Some(icon) = &item.icon {
                    has_icon = true;
                    let _ =
                        measure::<Renderer>(renderer, &icon.content, metrics.text_size, icon_font);
                }
                if let Some(shortcut) = &item.shortcut {
                    let s = shortcut.to_string();
                    let w = measure::<Renderer>(renderer, &s, metrics.shortcut_text_size, font);
                    row_w += w + metrics.gap * 2.0;
                }
                content_width = content_width.max(row_w);
            }
            Entry::Submenu(sub) => {
                let label_width =
                    measure::<Renderer>(renderer, &sub.label, metrics.text_size, font);
                content_width =
                    content_width.max(label_width + metrics.gap + metrics.submenu_indicator_width);
            }
            Entry::Separator => {}
        }
    }

    let icon_cell = if has_icon { metrics.icon_cell } else { 0.0 };
    let indicator_cell = metrics.submenu_indicator_width + metrics.gap;

    let viewport_max_width =
        (viewport.width - metrics.popup_padding.left - metrics.popup_padding.right).max(0.0);
    let width = (metrics.popup_padding.left
        + icon_cell
        + content_width
        + indicator_cell
        + metrics.popup_padding.right)
        .max(metrics.min_width)
        .min(viewport_max_width.max(metrics.min_width));

    let mut height = metrics.popup_padding.top + metrics.popup_padding.bottom;
    for entry in &menu.entries {
        height += match entry {
            Entry::Separator => metrics.separator_height,
            _ => metrics.row_height,
        };
    }

    let mut x = anchor_origin.x;
    let mut y = anchor_origin.y;
    if x + width > viewport.width {
        x = (viewport.width - width).max(0.0);
    }
    if y + height > viewport.height {
        y = (anchor_origin.y - height).max(0.0);
    }

    layout::Node::new(Size::new(width, height)).move_to(Point::new(x, y))
}

fn measure<Renderer: text::Renderer<Font = Font>>(
    _renderer: &Renderer,
    content: &str,
    size: f32,
    font: Font,
) -> f32 {
    let paragraph = Renderer::Paragraph::with_text(Text {
        content,
        bounds: Size::new(f32::INFINITY, f32::INFINITY),
        size: Pixels(size),
        line_height: text::LineHeight::default(),
        font,
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
    });
    paragraph.min_width()
}

/// Returns either the original `content` (if its rendered width fits
/// within `available_width`) or a truncated version ending with a
/// horizontal ellipsis (`…`) that does fit.
///
/// Returns an empty string if not even the ellipsis glyph fits.
fn fit_with_ellipsis<Renderer: text::Renderer<Font = Font>>(
    renderer: &Renderer,
    content: &str,
    available_width: f32,
    text_size: f32,
    font: Font,
) -> String {
    if available_width <= 0.0 {
        return String::new();
    }

    let full = measure::<Renderer>(renderer, content, text_size, font);
    if full <= available_width {
        return content.to_string();
    }

    const ELLIPSIS: char = '…';
    let ellipsis_width = measure::<Renderer>(renderer, "…", text_size, font);
    if ellipsis_width > available_width {
        return String::new();
    }

    // Binary search over character boundaries: find the largest prefix
    // `n` such that `content[..cut(n)] + "…"` still fits.
    let char_indices: Vec<usize> = content.char_indices().map(|(i, _)| i).collect();
    let mut lo = 0usize;
    let mut hi = char_indices.len();
    let mut best = String::from(ELLIPSIS);

    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if mid == 0 {
            break;
        }
        let cut = char_indices.get(mid).copied().unwrap_or(content.len());
        let mut candidate = String::with_capacity(cut + ELLIPSIS.len_utf8());
        candidate.push_str(&content[..cut]);
        candidate.push(ELLIPSIS);
        let w = measure::<Renderer>(renderer, &candidate, text_size, font);
        if w <= available_width {
            best = candidate;
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    best
}

#[allow(clippy::too_many_arguments)]
fn draw_menu<Message, Renderer>(
    renderer: &mut Renderer,
    menu: &Menu<Message>,
    layout: Layout<'_>,
    _cursor: mouse::Cursor,
    style: &Style,
    metrics: Metrics,
    font: Font,
    hovered: Option<usize>,
    opened: Option<usize>,
) where
    Renderer: text::Renderer<Font = Font>,
{
    let bounds = layout.bounds();

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: style.menu_border,
            shadow: style.menu_shadow,
            ..renderer::Quad::default()
        },
        style.menu_background,
    );

    let mut y = bounds.y + metrics.popup_padding.top;

    for (i, entry) in menu.entries.iter().enumerate() {
        if let Entry::Separator = entry {
            let line_y = y + metrics.separator_height / 2.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + metrics.popup_padding.left,
                        y: line_y,
                        width: bounds.width
                            - metrics.popup_padding.left
                            - metrics.popup_padding.right,
                        height: 1.0,
                    },
                    border: Border {
                        radius: 0.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..renderer::Quad::default()
                },
                style.separator_color,
            );
            y += metrics.separator_height;
            continue;
        }

        let highlighted = hovered == Some(i) || opened == Some(i);

        let row_bounds = Rectangle {
            x: bounds.x + 2.0,
            y,
            width: bounds.width - 4.0,
            height: metrics.row_height,
        };

        if highlighted {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: row_bounds,
                    border: Border {
                        radius: style.item_radius.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..renderer::Quad::default()
                },
                style.item_background_hovered,
            );
        }

        let text_y = y + (metrics.row_height - metrics.text_size) / 2.0;

        match entry {
            Entry::Item(item) => {
                let (text_color, shortcut_color) = if !item.enabled {
                    (style.item_text_disabled, style.item_text_disabled)
                } else if highlighted {
                    (style.item_text_hovered, style.item_text_hovered)
                } else {
                    (style.item_text, style.shortcut_text)
                };

                let label_x = bounds.x + metrics.popup_padding.left + metrics.icon_cell;

                if item.checked {
                    let check_color = if !item.enabled {
                        style.item_text_disabled
                    } else if highlighted {
                        style.item_text_hovered
                    } else {
                        style.check_color
                    };
                    renderer.fill_text(
                        Text {
                            content: style.check_glyph.to_string(),
                            bounds: Size::new(metrics.icon_cell, metrics.text_size),
                            size: Pixels(metrics.text_size),
                            line_height: text::LineHeight::default(),
                            font,
                            align_x: text::Alignment::Default,
                            align_y: alignment::Vertical::Top,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(bounds.x + metrics.popup_padding.left, text_y),
                        check_color,
                        bounds,
                    );
                } else if let Some(icon) = &item.icon {
                    let icon_font = icon.font.unwrap_or(font);
                    renderer.fill_text(
                        Text {
                            content: icon.content.clone(),
                            bounds: Size::new(metrics.icon_cell, metrics.text_size),
                            size: Pixels(metrics.text_size),
                            line_height: text::LineHeight::default(),
                            font: icon_font,
                            align_x: text::Alignment::Default,
                            align_y: alignment::Vertical::Top,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(bounds.x + metrics.popup_padding.left, text_y),
                        text_color,
                        bounds,
                    );
                }

                // Pre-measure the shortcut so we can bound the label draw
                // and avoid a label overflowing into the shortcut column.
                let shortcut_text = item.shortcut.as_ref().map(|s| s.to_string());
                let shortcut_width = shortcut_text.as_ref().map_or(0.0, |t| {
                    measure::<Renderer>(renderer, t, metrics.shortcut_text_size, font)
                });
                // Right edge of the reserved submenu-indicator column.
                let submenu_column_right = bounds.x + bounds.width - metrics.popup_padding.right;
                // Shortcut sits to the left of the reserved `▸` column,
                // with a `gap` of breathing room between them.
                let shortcut_right =
                    submenu_column_right - metrics.submenu_indicator_width - metrics.gap;
                let shortcut_column_left = if shortcut_width > 0.0 {
                    shortcut_right - shortcut_width - metrics.gap
                } else {
                    shortcut_right
                };
                let label_available = (shortcut_column_left - label_x).max(0.0);
                let label_display = fit_with_ellipsis::<Renderer>(
                    renderer,
                    &item.label,
                    label_available,
                    metrics.text_size,
                    font,
                );

                renderer.fill_text(
                    Text {
                        content: label_display,
                        bounds: Size::new(label_available, metrics.text_size),
                        size: Pixels(metrics.text_size),
                        line_height: text::LineHeight::default(),
                        font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(label_x, text_y),
                    text_color,
                    bounds,
                );

                if let Some(text) = shortcut_text {
                    let shortcut_text_y =
                        y + (metrics.row_height - metrics.shortcut_text_size) / 2.0;
                    renderer.fill_text(
                        Text {
                            content: text,
                            bounds: Size::new(shortcut_width, metrics.shortcut_text_size),
                            size: Pixels(metrics.shortcut_text_size),
                            line_height: text::LineHeight::default(),
                            font,
                            align_x: text::Alignment::Default,
                            align_y: alignment::Vertical::Top,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::None,
                        },
                        Point::new(shortcut_right - shortcut_width, shortcut_text_y),
                        shortcut_color,
                        bounds,
                    );
                }
            }
            Entry::Submenu(sub) => {
                let (text_color, arrow_color) = if highlighted {
                    (style.item_text_hovered, style.item_text_hovered)
                } else {
                    (style.item_text, style.shortcut_text)
                };
                let label_x = bounds.x + metrics.popup_padding.left + metrics.icon_cell;
                // Submenus have no shortcut, but still reserve the `▸`
                // column. Label is bounded by where that column starts.
                let submenu_column_left = bounds.x + bounds.width
                    - metrics.popup_padding.right
                    - metrics.submenu_indicator_width
                    - metrics.gap;
                let label_available = (submenu_column_left - label_x).max(0.0);
                let label_display = fit_with_ellipsis::<Renderer>(
                    renderer,
                    &sub.label,
                    label_available,
                    metrics.text_size,
                    font,
                );

                renderer.fill_text(
                    Text {
                        content: label_display,
                        bounds: Size::new(label_available, metrics.text_size),
                        size: Pixels(metrics.text_size),
                        line_height: text::LineHeight::default(),
                        font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(label_x, text_y),
                    text_color,
                    bounds,
                );

                renderer.fill_text(
                    Text {
                        content: "▸".to_string(),
                        bounds: Size::new(metrics.submenu_indicator_width, metrics.text_size),
                        size: Pixels(metrics.text_size),
                        line_height: text::LineHeight::default(),
                        font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(
                        bounds.x + bounds.width
                            - metrics.popup_padding.right
                            - metrics.submenu_indicator_width,
                        text_y,
                    ),
                    arrow_color,
                    bounds,
                );
            }
            Entry::Separator => unreachable!(),
        }

        y += metrics.row_height;
    }
}

#[cfg(test)]
mod tests {
    //! Regression tests for the menu overlay's hover-driven path
    //! decisions. These exercise [`compute_hover_update`] directly so
    //! they need no real iced runtime.

    use super::*;
    use crate::menu::item::{Item, Menu};

    #[derive(Clone, Debug, PartialEq)]
    struct Msg;

    fn test_metrics() -> Metrics {
        // Mirrors `Metrics::new` with values typical of the demo theme.
        Metrics::new(
            14.0,
            Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 8.0,
            },
            8.0,
        )
    }

    /// Regression test: when the cursor moves directly from the parent
    /// dropdown panel into an already-open second-level submenu panel,
    /// the second-level panel must remain open. Prior to the fix the
    /// rebuild logic truncated `open_path` because the cursor had left
    /// the parent panel, even though it had simultaneously entered the
    /// child panel.
    #[test]
    fn cursor_in_child_panel_keeps_submenu_open() {
        // Parent menu with a Submenu at index 2.
        let submenu: Menu<Msg> = Menu::new("More")
            .push(Item::new("Deep A"))
            .push(Item::new("Deep B"));
        let menus: Vec<Menu<Msg>> = vec![
            Menu::new("File")
                .push(Item::new("Open"))
                .push(Item::new("Save"))
                .submenu(submenu),
        ];

        let metrics = test_metrics();

        // Parent panel: row 0 = Open, row 1 = Save, row 2 = Submenu.
        let parent_bounds = Rectangle {
            x: 0.0,
            y: 20.0,
            width: 160.0,
            height: metrics.popup_padding.top
                + 3.0 * metrics.row_height
                + metrics.popup_padding.bottom,
        };
        // Child panel sits flush to the right of the parent and is
        // anchored vertically near the submenu trigger row.
        let child_bounds_rect = Rectangle {
            x: parent_bounds.x + parent_bounds.width,
            y: parent_bounds.y + metrics.popup_padding.top + 2.0 * metrics.row_height,
            width: 160.0,
            height: metrics.popup_padding.top
                + 2.0 * metrics.row_height
                + metrics.popup_padding.bottom,
        };
        let child_bounds = vec![parent_bounds, child_bounds_rect];

        // A simple bar with one label above the parent panel.
        let bar_label_bounds = vec![Rectangle {
            x: 0.0,
            y: 0.0,
            width: 60.0,
            height: 20.0,
        }];

        let bridges = hover_bridges(2, &child_bounds, &bar_label_bounds, 0, metrics);

        let path = vec![0, 2];
        // Previously the user was hovering the submenu trigger row.
        let prev_hover_path = vec![Some(2_usize), None];

        // Cursor sits squarely inside the child panel, on its first row.
        let cursor = Point::new(
            child_bounds_rect.x + 20.0,
            child_bounds_rect.y + metrics.popup_padding.top + metrics.row_height * 0.5,
        );

        let result = compute_hover_update(
            &menus,
            &path,
            &prev_hover_path,
            &child_bounds,
            &bar_label_bounds,
            &bridges,
            metrics,
            cursor,
        );

        // The submenu panel must remain open. Either Preserve (path
        // unchanged) or a Rebuild that keeps both depths is acceptable;
        // anything that drops the trailing `2` is the bug.
        let new_path: Vec<usize> = match result {
            HoverUpdate::Preserve { .. } => path.clone(),
            HoverUpdate::Rebuild { open_path, .. } => open_path,
            HoverUpdate::SwitchTopLevel(_) => panic!("unexpected SwitchTopLevel"),
        };
        assert_eq!(
            new_path, path,
            "moving the cursor into the second-level submenu panel must \
             not close it; got open_path = {new_path:?}"
        );
    }
}
