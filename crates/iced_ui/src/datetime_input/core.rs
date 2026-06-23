//! Internal core shared by [`DateInput`], [`TimeInput`], and
//! [`DateTimeInput`].
//!
//! All three widgets are thin public wrappers around the same
//! [`Picker`] widget: a text field (wrapping iced's built-in
//! `text_input`, following the [`NumberInput`](crate::NumberInput)
//! architecture) with a trailing trigger that opens a popup overlay
//! (following the [`ColorPicker`](crate::ColorPicker) architecture).
//! The popup shows a calendar grid, an hour/minute grid, or both,
//! depending on the [`Mode`].
//!
//! Values are carried internally as [`chrono::NaiveDateTime`]; the
//! wrappers convert from/to `NaiveDate` and `NaiveTime`.
//!
//! [`DateInput`]: crate::DateInput
//! [`TimeInput`]: crate::TimeInput
//! [`DateTimeInput`]: crate::DateTimeInput

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::text::{self, Renderer as _};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::widget::text_input::{self, TextInput};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, keyboard,
    mouse,
};

use super::style::{Style, StyleFn};
use super::{grid, style};
use crate::text_input::style::{self as input_style, Variant};
use crate::theme::Theme;

// -- Mode --

/// Which value components a [`Picker`] edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Calendar date (`YYYY-MM-DD`).
    Date,
    /// Time of day (`HH:MM`, 24-hour).
    Time,
    /// Date and time (`YYYY-MM-DD HH:MM`).
    DateTime,
}

impl Mode {
    fn format_str(self) -> &'static str {
        match self {
            Self::Date => "%Y-%m-%d",
            Self::Time => "%H:%M",
            Self::DateTime => "%Y-%m-%d %H:%M",
        }
    }

    fn max_len(self) -> usize {
        match self {
            Self::Date => 10,
            Self::Time => 5,
            Self::DateTime => 16,
        }
    }

    fn is_allowed_char(self, c: char) -> bool {
        c.is_ascii_digit()
            || match self {
                Self::Date => c == '-',
                Self::Time => c == ':',
                Self::DateTime => c == '-' || c == ':' || c == ' ',
            }
    }

    fn has_calendar(self) -> bool {
        matches!(self, Self::Date | Self::DateTime)
    }

    fn has_time(self) -> bool {
        matches!(self, Self::Time | Self::DateTime)
    }

    fn default_width(self) -> f32 {
        match self {
            Self::Date => 150.0,
            Self::Time => 110.0,
            Self::DateTime => 210.0,
        }
    }

    fn format(self, dt: NaiveDateTime) -> String {
        dt.format(self.format_str()).to_string()
    }

    /// Parses `text` into a full date-time, taking the missing
    /// component (if any) from `base`.
    fn parse(self, text: &str, base: NaiveDateTime) -> Option<NaiveDateTime> {
        match self {
            Self::Date => NaiveDate::parse_from_str(text, self.format_str())
                .ok()
                .map(|d| d.and_time(base.time())),
            Self::Time => NaiveTime::parse_from_str(text, self.format_str())
                .ok()
                .map(|t| base.date().and_time(t)),
            Self::DateTime => NaiveDateTime::parse_from_str(text, self.format_str()).ok(),
        }
    }
}

// -- Geometry constants --

/// Width of the trailing trigger area inside the field.
const TRIGGER_WIDTH: f32 = 28.0;
/// Horizontal padding inside the field.
const PADDING_H: f32 = 12.0;
/// Vertical padding inside the field.
const PADDING_V: f32 = 8.0;

/// Padding inside the popup panel.
const POPUP_PADDING: f32 = 8.0;
/// Width of a calendar day cell.
const CELL_W: f32 = 30.0;
/// Height of a calendar day cell.
const CELL_H: f32 = 26.0;
/// Width of the popup content area.
const CONTENT_WIDTH: f32 = CELL_W * grid::GRID_COLS as f32;
/// Height of the month navigation header row.
const HEADER_H: f32 = 28.0;
/// Width of the previous/next month chevron hit areas.
const CHEVRON_W: f32 = 28.0;
/// Height of the weekday label row.
const WEEKDAY_H: f32 = 20.0;
/// Vertical gap between popup sections.
const SECTION_GAP: f32 = 8.0;
/// Height of the "Hour"/"Minute" column labels.
const TIME_LABEL_H: f32 = 16.0;
/// Height of one hour/minute list item.
const TIME_ITEM_H: f32 = 24.0;
/// Number of items visible at once in the hour/minute columns.
const TIME_VISIBLE_ITEMS: usize = 7;
/// Number of selectable hours.
const HOUR_COUNT: usize = 24;
/// Width of the scrollbar at the right edge of a time column.
const SCROLLBAR_W: f32 = 4.0;
/// Inset of the scrollbar from the column edges.
const SCROLLBAR_MARGIN: f32 = 2.0;
/// Minimum height of a scrollbar thumb.
const SCROLLBAR_MIN_THUMB: f32 = 20.0;
/// Vertical distance a pressed time column must move before the press
/// becomes a content drag instead of a click.
const DRAG_THRESHOLD: f32 = 5.0;
/// Gap between the field and the popup.
const POPUP_OFFSET: f32 = 4.0;

fn calendar_height() -> f32 {
    HEADER_H + WEEKDAY_H + grid::GRID_ROWS as f32 * CELL_H
}

/// Height of the visible part of an hour/minute column.
fn time_viewport_height() -> f32 {
    TIME_VISIBLE_ITEMS as f32 * TIME_ITEM_H
}

fn time_height() -> f32 {
    TIME_LABEL_H + time_viewport_height()
}

/// Number of selectable minutes for the given step.
fn minute_count(step: u32) -> usize {
    60usize.div_ceil(step as usize)
}

fn popup_size(mode: Mode) -> Size {
    let mut content_h = 0.0;
    if mode.has_calendar() {
        content_h += calendar_height();
    }
    if mode.has_time() {
        if mode.has_calendar() {
            content_h += SECTION_GAP;
        }
        content_h += time_height();
    }
    Size::new(
        CONTENT_WIDTH + POPUP_PADDING * 2.0,
        content_h + POPUP_PADDING * 2.0,
    )
}

/// Regions of the calendar section, in absolute coordinates.
struct CalendarRegions {
    prev: Rectangle,
    title: Rectangle,
    next: Rectangle,
    weekdays: Rectangle,
    grid: Rectangle,
}

fn calendar_regions(origin: Point) -> CalendarRegions {
    let prev = Rectangle::new(origin, Size::new(CHEVRON_W, HEADER_H));
    let next = Rectangle::new(
        Point::new(origin.x + CONTENT_WIDTH - CHEVRON_W, origin.y),
        Size::new(CHEVRON_W, HEADER_H),
    );
    let title = Rectangle::new(
        Point::new(origin.x + CHEVRON_W, origin.y),
        Size::new(CONTENT_WIDTH - CHEVRON_W * 2.0, HEADER_H),
    );
    let weekdays = Rectangle::new(
        Point::new(origin.x, origin.y + HEADER_H),
        Size::new(CONTENT_WIDTH, WEEKDAY_H),
    );
    let grid = Rectangle::new(
        Point::new(origin.x, origin.y + HEADER_H + WEEKDAY_H),
        Size::new(CONTENT_WIDTH, grid::GRID_ROWS as f32 * CELL_H),
    );

    CalendarRegions {
        prev,
        title,
        next,
        weekdays,
        grid,
    }
}

/// Splits the header title into the clickable month and year segments
/// shown in the day view.
fn title_segments(title: Rectangle) -> (Rectangle, Rectangle) {
    let month_w = title.width * 0.58;
    let month = Rectangle {
        width: month_w,
        ..title
    };
    let year = Rectangle {
        x: title.x + month_w,
        width: title.width - month_w,
        ..title
    };
    (month, year)
}

/// The body area below the header (weekday row + day grid), reused by
/// the month and year list views.
fn calendar_body(origin: Point) -> Rectangle {
    Rectangle::new(
        Point::new(origin.x, origin.y + HEADER_H),
        Size::new(CONTENT_WIDTH, WEEKDAY_H + grid::GRID_ROWS as f32 * CELL_H),
    )
}

/// Regions of the time section, in absolute coordinates.
struct TimeRegions {
    hour_label: Rectangle,
    minute_label: Rectangle,
    hour_col: Rectangle,
    minute_col: Rectangle,
}

fn time_regions(origin: Point) -> TimeRegions {
    let col_w = CONTENT_WIDTH / 2.0;

    let hour_label = Rectangle::new(origin, Size::new(col_w, TIME_LABEL_H));
    let minute_label = Rectangle::new(
        Point::new(origin.x + col_w, origin.y),
        Size::new(col_w, TIME_LABEL_H),
    );

    let cols_y = origin.y + TIME_LABEL_H;
    let col_size = Size::new(col_w, time_viewport_height());
    let hour_col = Rectangle::new(Point::new(origin.x, cols_y), col_size);
    let minute_col = Rectangle::new(Point::new(origin.x + col_w, cols_y), col_size);

    TimeRegions {
        hour_label,
        minute_label,
        hour_col,
        minute_col,
    }
}

/// Scrollbar track and thumb of a time column, or `None` when the
/// content fits without scrolling.
struct Scrollbar {
    track: Rectangle,
    thumb: Rectangle,
}

fn scrollbar(col: Rectangle, count: usize, scroll: f32) -> Option<Scrollbar> {
    let content = count as f32 * TIME_ITEM_H;
    if content <= col.height {
        return None;
    }

    let track = Rectangle {
        x: col.x + col.width - SCROLLBAR_W - SCROLLBAR_MARGIN,
        y: col.y + SCROLLBAR_MARGIN,
        width: SCROLLBAR_W,
        height: col.height - SCROLLBAR_MARGIN * 2.0,
    };

    let thumb_h = (track.height * col.height / content).max(SCROLLBAR_MIN_THUMB);
    let fraction = (scroll / (content - col.height)).clamp(0.0, 1.0);
    let thumb = Rectangle {
        y: track.y + fraction * (track.height - thumb_h),
        height: thumb_h,
        ..track
    };

    Some(Scrollbar { track, thumb })
}

/// Maximum scroll offset of a column with `count` items.
fn max_scroll(count: usize) -> f32 {
    (count as f32 * TIME_ITEM_H - time_viewport_height()).max(0.0)
}

/// The scroll offset that vertically centers `index` in the column.
fn centered_scroll(index: usize, count: usize) -> f32 {
    let target = index as f32 * TIME_ITEM_H - (time_viewport_height() - TIME_ITEM_H) / 2.0;
    target.clamp(0.0, max_scroll(count))
}

/// The trailing trigger area of the field.
fn trigger_bounds(field: Rectangle) -> Rectangle {
    Rectangle {
        x: field.x + field.width - TRIGGER_WIDTH,
        width: TRIGGER_WIDTH,
        ..field
    }
}

fn inset(r: Rectangle, d: f32) -> Rectangle {
    Rectangle {
        x: r.x + d,
        y: r.y + d,
        width: (r.width - d * 2.0).max(0.0),
        height: (r.height - d * 2.0).max(0.0),
    }
}

// -- Internal field machinery --

/// Internal message type for the wrapped text_input.
#[derive(Debug, Clone)]
enum InternalMsg {
    TextChanged(String),
    Submit,
}

/// Which view the calendar section is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalendarView {
    /// The regular day grid.
    Days,
    /// A 3 × 4 list of months of the view year.
    Months,
    /// A 4 × 5 page of years.
    Years,
}

/// One of the two scrollable time columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeColumn {
    Hour,
    Minute,
}

/// An interactive target of the calendar section, resolved from a
/// cursor position and the current [`CalendarView`].
///
/// Selection fires on mouse release: grid cells activate the cell
/// under the release position (so a press can be dragged onto the
/// desired day), while header buttons require press and release on
/// the same control (standard button semantics). See
/// [`release_activates`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PressTarget {
    /// The previous month/year/page chevron.
    PrevChevron,
    /// The next month/year/page chevron.
    NextChevron,
    /// The month segment of the day-view title.
    TitleMonth,
    /// The year segment of the day-view title.
    TitleYear,
    /// The whole title (month view; opens the year list).
    Title,
    /// A cell of the day grid.
    DayCell(usize),
    /// A cell of the month list.
    MonthCell(usize),
    /// A cell of the year list.
    YearCell(usize),
}

/// Whether releasing over `released` after pressing on `pressed`
/// activates a target (and which one: always the released target).
///
/// Grid cells of the same kind are interchangeable — a press may be
/// dragged across the grid and the cell under the release wins.
/// Everything else (chevrons, title segments) behaves like a regular
/// button: press and release must land on the same control.
fn release_activates(pressed: PressTarget, released: PressTarget) -> bool {
    use PressTarget::{DayCell, MonthCell, YearCell};

    matches!(
        (pressed, released),
        (DayCell(_), DayCell(_)) | (MonthCell(_), MonthCell(_)) | (YearCell(_), YearCell(_))
    ) || pressed == released
}

/// An in-progress left-button press inside the popup.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pressed {
    /// A calendar/header target was pressed; it fires on release (see
    /// [`release_activates`] for the matching rules).
    Target(PressTarget),
    /// A scrollbar thumb drag: the column and the cursor's offset from
    /// the top of the thumb at grab time.
    Scrollbar(TimeColumn, f32),
    /// A press in a time column's items area. Stays a click (selecting
    /// on release) unless the cursor moves beyond [`DRAG_THRESHOLD`].
    TimePress {
        column: TimeColumn,
        start_y: f32,
        start_scroll: f32,
    },
    /// A content drag in a time column: the content follows the
    /// cursor; releasing selects nothing.
    TimeDrag {
        column: TimeColumn,
        start_y: f32,
        start_scroll: f32,
    },
}

/// Internal state stored in the widget tree.
struct State {
    /// The current text displayed in the input.
    text: String,
    /// Whether the current text is a valid in-range value.
    is_valid: bool,
    /// Whether the popup overlay is open.
    open: bool,
    /// Which view the calendar section is showing.
    view: CalendarView,
    /// The year of the month shown in the calendar grid.
    view_year: i32,
    /// The month (1–12) shown in the calendar grid.
    view_month: u32,
    /// The value being edited in the popup.
    edit: NaiveDateTime,
    /// The date marked as "today" in the calendar grid. Captured when
    /// the popup opens so it stays stable for the whole session.
    today: NaiveDate,
    /// Scroll offset of the hour column, in pixels.
    hour_scroll: f32,
    /// Scroll offset of the minute column, in pixels.
    minute_scroll: f32,
    /// The in-progress left-button press, if any.
    pressed: Option<Pressed>,
}

impl State {
    fn scroll(&self, column: TimeColumn) -> f32 {
        match column {
            TimeColumn::Hour => self.hour_scroll,
            TimeColumn::Minute => self.minute_scroll,
        }
    }

    fn scroll_mut(&mut self, column: TimeColumn) -> &mut f32 {
        match column {
            TimeColumn::Hour => &mut self.hour_scroll,
            TimeColumn::Minute => &mut self.minute_scroll,
        }
    }

    /// Whether a time-column drag (scrollbar or content) is active.
    fn is_time_dragging(&self) -> bool {
        matches!(
            self.pressed,
            Some(Pressed::Scrollbar(..) | Pressed::TimeDrag { .. })
        )
    }
}

/// Builds the inner iced text_input widget using the given text.
fn build_inner_text_input<'a>(
    text: &'a str,
    variant: Variant,
) -> TextInput<'a, InternalMsg, Theme, iced::Renderer> {
    TextInput::new("", text)
        .width(Length::Fill)
        .padding([4, 0])
        .on_input(InternalMsg::TextChanged)
        .on_submit(InternalMsg::Submit)
        .style(move |theme: &Theme, status| {
            let our_status = match status {
                text_input::Status::Active => input_style::Status::Active,
                text_input::Status::Hovered => input_style::Status::Hovered,
                text_input::Status::Focused { .. } => input_style::Status::Focused,
                text_input::Status::Disabled => input_style::Status::Disabled,
            };
            let our_style = match variant {
                Variant::Outlined => input_style::outlined(theme, our_status),
                Variant::Filled => input_style::filled(theme, our_status),
            };
            // Inner text_input is transparent — the outer widget draws
            // the background/border.
            text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                icon: our_style.icon_color,
                placeholder: our_style.placeholder_color,
                value: our_style.value_color,
                selection: our_style.selection_color,
            }
        })
}

type Paragraph = <iced::Renderer as text::Renderer>::Paragraph;

// -- Picker widget --

/// The shared date/time picker widget.
///
/// Not exposed directly: [`DateInput`](crate::DateInput),
/// [`TimeInput`](crate::TimeInput), and
/// [`DateTimeInput`](crate::DateTimeInput) wrap it with typed APIs.
pub(crate) struct Picker<'a, Message> {
    mode: Mode,
    value: NaiveDateTime,
    on_change: Option<Box<dyn Fn(NaiveDateTime) -> Message + 'a>>,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    today: Option<NaiveDate>,
    minute_step: u32,
    width: Length,
    variant: Variant,
    class: StyleFn<'a, Theme>,
}

impl<'a, Message> Picker<'a, Message> {
    pub(crate) fn new(mode: Mode, value: NaiveDateTime) -> Self {
        Self {
            mode,
            value,
            on_change: None,
            min: None,
            max: None,
            today: None,
            minute_step: 1,
            width: Length::Fixed(mode.default_width()),
            variant: Variant::default(),
            class: Box::new(style::default),
        }
    }

    pub(crate) fn on_change(mut self, f: Box<dyn Fn(NaiveDateTime) -> Message + 'a>) -> Self {
        self.on_change = Some(f);
        self
    }

    pub(crate) fn min(mut self, min: NaiveDateTime) -> Self {
        self.min = Some(min);
        self
    }

    pub(crate) fn max(mut self, max: NaiveDateTime) -> Self {
        self.max = Some(max);
        self
    }

    pub(crate) fn today(mut self, today: NaiveDate) -> Self {
        self.today = Some(today);
        self
    }

    pub(crate) fn minute_step(mut self, step: u32) -> Self {
        self.minute_step = step.clamp(1, 59);
        self
    }

    /// Initial scroll offsets centering `edit`'s hour and minute in
    /// their columns.
    fn initial_scrolls(&self, edit: NaiveDateTime) -> (f32, f32) {
        let hour = centered_scroll(edit.time().hour() as usize, HOUR_COUNT);
        let count = minute_count(self.minute_step);
        let index = ((edit.time().minute() / self.minute_step) as usize).min(count - 1);
        (hour, centered_scroll(index, count))
    }

    /// The date to mark as "today": the override if set, otherwise the
    /// system's local date.
    fn today_or_now(&self) -> NaiveDate {
        self.today.unwrap_or_else(|| Local::now().date_naive())
    }

    pub(crate) fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub(crate) fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub(crate) fn style(mut self, class: StyleFn<'a, Theme>) -> Self {
        self.class = class;
        self
    }

    fn clamp(&self, dt: NaiveDateTime) -> NaiveDateTime {
        clamp(dt, self.min, self.max)
    }

    fn in_range(&self, dt: NaiveDateTime) -> bool {
        self.min.is_none_or(|min| dt >= min) && self.max.is_none_or(|max| dt <= max)
    }

    /// Validates the entire string: allowed characters and length.
    fn is_valid_text(&self, text: &str) -> bool {
        text.len() <= self.mode.max_len() && text.chars().all(|c| self.mode.is_allowed_char(c))
    }
}

fn clamp(
    dt: NaiveDateTime,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
) -> NaiveDateTime {
    let mut dt = dt;
    if let Some(min) = min
        && dt < min
    {
        dt = min;
    }
    if let Some(max) = max
        && dt > max
    {
        dt = max;
    }
    dt
}

impl<'a, Message> Widget<Message, Theme, iced::Renderer> for Picker<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let (hour_scroll, minute_scroll) = self.initial_scrolls(self.value);
        tree::State::new(State {
            text: self.mode.format(self.value),
            is_valid: true,
            open: false,
            view: CalendarView::Days,
            view_year: self.value.date().year(),
            view_month: self.value.date().month(),
            edit: self.value,
            today: self.today_or_now(),
            hour_scroll,
            minute_scroll,
            pressed: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        let formatted = self.mode.format(self.value);
        let inner = build_inner_text_input(&formatted, self.variant);
        vec![Tree::new(
            &inner as &dyn Widget<InternalMsg, Theme, iced::Renderer>,
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Check if the text_input is focused
        let is_focused = tree
            .children
            .first()
            .map(|child| {
                child
                    .state
                    .downcast_ref::<text_input::State<Paragraph>>()
                    .is_focused()
            })
            .unwrap_or(false);

        // If not focused, sync text with the current value
        if !is_focused {
            let formatted = self.mode.format(self.value);
            if state.text != formatted {
                state.text = formatted;
                state.is_valid = true;
            }
        }

        // Ensure children vector has one entry
        if tree.children.is_empty() {
            let inner = build_inner_text_input(&state.text, self.variant);
            tree.children.push(Tree::new(
                &inner as &dyn Widget<InternalMsg, Theme, iced::Renderer>,
            ));
        }
        // Do NOT diff the child text_input — same as the NumberInput
        // (and iced ComboBox) pattern.
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
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<State>();

        // Build text_input and lay it out
        let text_snapshot = state.text.clone();
        let mut inner = build_inner_text_input(&text_snapshot, self.variant);

        let inner_limits = limits.width(self.width);
        let text_input_max_width =
            (inner_limits.max().width - TRIGGER_WIDTH - PADDING_H * 2.0).max(0.0);

        let text_input_limits = layout::Limits::new(
            Size::ZERO,
            Size::new(text_input_max_width, inner_limits.max().height),
        )
        .width(Length::Fill);

        let mut text_node = inner.layout(&mut tree.children[0], renderer, &text_input_limits, None);

        let height = text_node.bounds().height + PADDING_V * 2.0;

        // Position the text_input child inside padding
        text_node = text_node.move_to(Point::new(PADDING_H, PADDING_V));

        // Compute total width
        let total_width = match self.width {
            Length::Fixed(w) => w,
            Length::Fill | Length::FillPortion(_) => inner_limits.max().width,
            Length::Shrink => text_node.bounds().width + TRIGGER_WIDTH + PADDING_H * 2.0,
        };

        layout::Node::with_children(Size::new(total_width, height), vec![text_node])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Detect focus state before processing the event
        let was_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .is_focused();

        // Handle trigger clicks (open the popup)
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(trigger_bounds(bounds))
        {
            let state = tree.state.downcast_mut::<State>();
            if !state.open {
                state.open = true;
                let parsed = self.mode.parse(&state.text, self.value);
                state.edit = self.clamp(parsed.unwrap_or(self.value));
                state.view = CalendarView::Days;
                state.view_year = state.edit.date().year();
                state.view_month = state.edit.date().month();
                state.today = self.today_or_now();
                let (hour_scroll, minute_scroll) = self.initial_scrolls(state.edit);
                state.hour_scroll = hour_scroll;
                state.minute_scroll = minute_scroll;
                state.pressed = None;
                shell.invalidate_layout();
                shell.request_redraw();
            }
            shell.capture_event();
            return;
        }

        // Clone the text to avoid borrow conflict: we need to create the
        // text_input from &str, but later mutate state.text based on messages.
        let text_snapshot = tree.state.downcast_ref::<State>().text.clone();

        // Forward event to the inner text_input using a local shell
        let mut local_messages: Vec<InternalMsg> = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let text_input_layout = layout.children().next().unwrap();

        let mut inner = build_inner_text_input(&text_snapshot, self.variant);
        inner.update(
            &mut tree.children[0],
            event,
            text_input_layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        // Forward shell state
        if local_shell.is_event_captured() {
            shell.capture_event();
        }
        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());

        // Now we can mutate state freely
        let state = tree.state.downcast_mut::<State>();

        // Process intercepted messages
        for msg in local_messages {
            match msg {
                InternalMsg::TextChanged(new_text) => {
                    // Character-level filtering
                    if !self.is_valid_text(&new_text) {
                        // Reject: don't update state.text. Next frame, the
                        // text_input will receive the old text and revert.
                        shell.request_redraw();
                        continue;
                    }

                    // Accept the text
                    state.text = new_text;

                    // Try to parse and validate
                    if state.text.is_empty() {
                        state.is_valid = true; // empty is acceptable while typing
                    } else if let Some(parsed) = self.mode.parse(&state.text, self.value) {
                        if self.in_range(parsed) {
                            state.is_valid = true;
                            state.edit = parsed;
                            state.view_year = parsed.date().year();
                            state.view_month = parsed.date().month();
                            if let Some(on_change) = &self.on_change {
                                shell.publish((on_change)(parsed));
                            }
                        } else {
                            state.is_valid = false;
                        }
                    } else {
                        // Intermediate state (e.g. "2026-0")
                        state.is_valid = false;
                    }
                    shell.request_redraw();
                }
                InternalMsg::Submit => {
                    // On Enter: clamp and commit
                    if let Some(parsed) = self.mode.parse(&state.text, self.value) {
                        let clamped = self.clamp(parsed);
                        state.text = self.mode.format(clamped);
                        state.is_valid = true;
                        if let Some(on_change) = &self.on_change {
                            shell.publish((on_change)(clamped));
                        }
                    } else {
                        // Revert to current value
                        state.text = self.mode.format(self.value);
                        state.is_valid = true;
                    }
                    shell.request_redraw();
                }
            }
        }

        // Detect blur (was focused, now isn't)
        let is_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .is_focused();

        if was_focused && !is_focused {
            let state = tree.state.downcast_mut::<State>();
            if let Some(parsed) = self.mode.parse(&state.text, self.value) {
                let clamped = self.clamp(parsed);
                state.text = self.mode.format(clamped);
                state.is_valid = true;
                if let Some(on_change) = &self.on_change {
                    shell.publish((on_change)(clamped));
                }
            } else {
                // Invalid text — revert
                state.text = self.mode.format(self.value);
                state.is_valid = true;
            }
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Determine focus state
        let is_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Paragraph>>()
            .is_focused();

        let palette = theme.extended_palette();
        let roundness = theme.radius(crate::Roundness::sx(1.0));

        // Compute outer container style
        let (bg, border_color, border_width) = if !state.is_valid {
            let bg = match self.variant {
                Variant::Outlined => Color::TRANSPARENT,
                Variant::Filled => palette.background.neutral.color,
            };
            (bg, palette.danger.base.color, 1.5)
        } else if is_focused || state.open {
            let bg = match self.variant {
                Variant::Outlined => Color::TRANSPARENT,
                Variant::Filled => palette.background.neutral.color,
            };
            (bg, palette.primary.base.color, 1.5)
        } else {
            match self.variant {
                Variant::Outlined => (Color::TRANSPARENT, palette.background.strong.color, 1.0),
                Variant::Filled => (palette.background.neutral.color, Color::TRANSPARENT, 0.0),
            }
        };

        // Draw outer background + border
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: border_color,
                    width: border_width,
                    radius: roundness.into(),
                },
                shadow: iced::Shadow::default(),
                ..renderer::Quad::default()
            },
            Background::Color(bg),
        );

        // Draw the inner text_input
        let text_input_layout = layout.children().next().unwrap();
        let inner = build_inner_text_input(&state.text, self.variant);
        inner.draw(
            &tree.children[0],
            renderer,
            theme,
            text_input_layout,
            cursor,
            None,
            viewport,
        );

        // Draw the trigger icon
        let icon_color = palette.background.strongest.color;
        let trigger = trigger_bounds(bounds);
        if self.mode.has_calendar() {
            draw_calendar_icon(renderer, trigger, icon_color);
        } else {
            draw_clock_icon(renderer, trigger, icon_color);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        if cursor.is_over(bounds) {
            // Pointer cursor over the trigger
            if cursor.is_over(trigger_bounds(bounds)) {
                return mouse::Interaction::Pointer;
            }

            // Delegate to text_input for the text area
            let text_input_layout = layout.children().next().unwrap();
            let state = tree.state.downcast_ref::<State>();
            let inner = build_inner_text_input(&state.text, self.variant);
            return inner.mouse_interaction(
                &tree.children[0],
                text_input_layout,
                cursor,
                viewport,
                renderer,
            );
        }

        mouse::Interaction::default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        if !state.open {
            return None;
        }

        let mut field_bounds = layout.bounds();
        field_bounds.x += translation.x;
        field_bounds.y += translation.y;

        Some(overlay::Element::new(Box::new(PickerOverlay {
            mode: self.mode,
            on_change: &self.on_change,
            class: &self.class,
            min: self.min,
            max: self.max,
            minute_step: self.minute_step,
            state,
            field_bounds,
        })))
    }
}

// -- Overlay --

struct PickerOverlay<'a, 'b, Message> {
    mode: Mode,
    on_change: &'b Option<Box<dyn Fn(NaiveDateTime) -> Message + 'a>>,
    class: &'b StyleFn<'a, Theme>,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    minute_step: u32,
    state: &'b mut State,
    field_bounds: Rectangle,
}

impl<'a, 'b, Message> PickerOverlay<'a, 'b, Message> {
    /// Origin of the calendar section content.
    fn content_origin(&self, bounds: Rectangle) -> Point {
        Point::new(bounds.x + POPUP_PADDING, bounds.y + POPUP_PADDING)
    }

    /// Origin of the time section content.
    fn time_origin(&self, bounds: Rectangle) -> Point {
        let origin = self.content_origin(bounds);
        if self.mode.has_calendar() {
            Point::new(origin.x, origin.y + calendar_height() + SECTION_GAP)
        } else {
            origin
        }
    }

    /// Whether a calendar day is selectable under the configured range.
    fn date_enabled(&self, date: NaiveDate) -> bool {
        self.min.is_none_or(|min| date >= min.date())
            && self.max.is_none_or(|max| date <= max.date())
    }

    /// Whether any day of `(year, month)` is within the configured
    /// range (and the month is representable at all).
    fn month_enabled(&self, year: i32, month: u32) -> bool {
        let Some(start) = NaiveDate::from_ymd_opt(year, month, 1) else {
            return false;
        };
        let (next_y, next_m) = grid::next_month(year, month);
        let Some(end) = NaiveDate::from_ymd_opt(next_y, next_m, 1).and_then(|d| d.pred_opt())
        else {
            return false;
        };

        self.min.is_none_or(|min| end >= min.date())
            && self.max.is_none_or(|max| start <= max.date())
    }

    /// Whether any day of `year` is within the configured range (and
    /// the year is representable at all).
    fn year_enabled(&self, year: i32) -> bool {
        let (Some(start), Some(end)) = (
            NaiveDate::from_ymd_opt(year, 1, 1),
            NaiveDate::from_ymd_opt(year, 12, 31),
        ) else {
            return false;
        };

        self.min.is_none_or(|min| end >= min.date())
            && self.max.is_none_or(|max| start <= max.date())
    }

    /// Number of items in a time column.
    fn item_count(&self, column: TimeColumn) -> usize {
        match column {
            TimeColumn::Hour => HOUR_COUNT,
            TimeColumn::Minute => minute_count(self.minute_step),
        }
    }

    /// Index of the currently selected item in a time column, if the
    /// current value is on the column's step.
    fn selected_index(&self, column: TimeColumn) -> Option<usize> {
        match column {
            TimeColumn::Hour => Some(self.state.edit.time().hour() as usize),
            TimeColumn::Minute => {
                let minute = self.state.edit.time().minute();
                minute
                    .is_multiple_of(self.minute_step)
                    .then_some((minute / self.minute_step) as usize)
            }
        }
    }

    /// Sets a column's scroll offset, clamped to its content.
    fn set_scroll(&mut self, column: TimeColumn, value: f32) {
        *self.state.scroll_mut(column) = value.clamp(0.0, max_scroll(self.item_count(column)));
    }

    /// Selects the item at `index` in a time column and commits.
    fn pick_time(&mut self, column: TimeColumn, index: usize, shell: &mut Shell<'_, Message>) {
        let time = self.state.edit.time();
        let (hour, minute) = match column {
            TimeColumn::Hour => (index as u32, time.minute()),
            TimeColumn::Minute => (time.hour(), index as u32 * self.minute_step),
        };
        self.state.edit = self
            .state
            .edit
            .date()
            .and_hms_opt(hour, minute, 0)
            .expect("hour in 0..24 and minute in 0..60");
        self.commit(shell);
    }

    /// Commits the edited value: clamps it, mirrors it into the field
    /// text, and publishes `on_change`.
    fn commit(&mut self, shell: &mut Shell<'_, Message>) {
        self.state.edit = clamp(self.state.edit, self.min, self.max);
        self.state.text = self.mode.format(self.state.edit);
        self.state.is_valid = true;
        if let Some(on_change) = self.on_change {
            shell.publish(on_change(self.state.edit));
        }
    }

    fn close(&mut self, shell: &mut Shell<'_, Message>) {
        self.state.open = false;
        shell.request_redraw();
    }

    /// Resolves the interactive calendar target under `pos`, given the
    /// current view. Used symmetrically on press and release so a
    /// target only activates when both land on it.
    fn calendar_target(&self, pos: Point, bounds: Rectangle) -> Option<PressTarget> {
        let origin = self.content_origin(bounds);
        let regions = calendar_regions(origin);
        let body = calendar_body(origin);

        if regions.prev.contains(pos) {
            return Some(PressTarget::PrevChevron);
        }
        if regions.next.contains(pos) {
            return Some(PressTarget::NextChevron);
        }

        match self.state.view {
            CalendarView::Days => {
                let (month_seg, year_seg) = title_segments(regions.title);
                if month_seg.contains(pos) {
                    return Some(PressTarget::TitleMonth);
                }
                if year_seg.contains(pos) {
                    return Some(PressTarget::TitleYear);
                }
                grid::cell_at(pos, regions.grid, grid::GRID_COLS, grid::GRID_ROWS)
                    .map(PressTarget::DayCell)
            }
            CalendarView::Months => {
                if regions.title.contains(pos) {
                    return Some(PressTarget::Title);
                }
                grid::cell_at(pos, body, grid::MONTH_COLS, grid::MONTH_ROWS)
                    .map(PressTarget::MonthCell)
            }
            CalendarView::Years => grid::cell_at(pos, body, grid::YEAR_COLS, grid::YEAR_ROWS)
                .map(PressTarget::YearCell),
        }
    }

    /// Performs the action of a calendar target (on release).
    fn activate_target(&mut self, target: PressTarget, shell: &mut Shell<'_, Message>) {
        match target {
            PressTarget::PrevChevron => match self.state.view {
                CalendarView::Days => {
                    let (y, m) = grid::prev_month(self.state.view_year, self.state.view_month);
                    self.state.view_year = y;
                    self.state.view_month = m;
                }
                CalendarView::Months => {
                    self.state.view_year = self.state.view_year.saturating_sub(1);
                }
                CalendarView::Years => {
                    self.state.view_year =
                        self.state.view_year.saturating_sub(grid::YEARS_PER_PAGE);
                }
            },
            PressTarget::NextChevron => match self.state.view {
                CalendarView::Days => {
                    let (y, m) = grid::next_month(self.state.view_year, self.state.view_month);
                    self.state.view_year = y;
                    self.state.view_month = m;
                }
                CalendarView::Months => {
                    self.state.view_year = self.state.view_year.saturating_add(1);
                }
                CalendarView::Years => {
                    self.state.view_year =
                        self.state.view_year.saturating_add(grid::YEARS_PER_PAGE);
                }
            },
            PressTarget::TitleMonth => self.state.view = CalendarView::Months,
            PressTarget::TitleYear | PressTarget::Title => self.state.view = CalendarView::Years,
            PressTarget::DayCell(index) => {
                let date = grid::grid_dates(self.state.view_year, self.state.view_month)[index];
                if self.date_enabled(date) {
                    self.state.edit = date.and_time(self.state.edit.time());
                    self.state.view_year = date.year();
                    self.state.view_month = date.month();
                    self.commit(shell);
                    if self.mode == Mode::Date {
                        self.close(shell);
                    }
                }
            }
            PressTarget::MonthCell(index) => {
                let month = index as u32 + 1;
                if self.month_enabled(self.state.view_year, month) {
                    self.state.view_month = month;
                    self.state.view = CalendarView::Days;
                }
            }
            PressTarget::YearCell(index) => {
                let year = grid::year_page_start(self.state.view_year) + index as i32;
                if self.year_enabled(year) {
                    self.state.view_year = year;
                    self.state.view = CalendarView::Days;
                }
            }
        }
    }

    /// Handles a left press at `pos` inside the time section: starts a
    /// scrollbar drag or arms an item press (which selects on release
    /// or becomes a content drag once moved).
    fn time_press(&mut self, pos: Point, bounds: Rectangle) {
        let regions = time_regions(self.time_origin(bounds));

        for (column, col) in [
            (TimeColumn::Hour, regions.hour_col),
            (TimeColumn::Minute, regions.minute_col),
        ] {
            if !col.contains(pos) {
                continue;
            }

            let count = self.item_count(column);
            let scroll = self.state.scroll(column);

            if let Some(sb) = scrollbar(col, count, scroll)
                && sb.track.contains(pos)
            {
                // Grab the thumb where it was pressed; pressing the
                // track instead jumps the thumb's center to the cursor.
                let grab = if sb.thumb.contains(pos) {
                    pos.y - sb.thumb.y
                } else {
                    sb.thumb.height / 2.0
                };
                self.state.pressed = Some(Pressed::Scrollbar(column, grab));
                self.drag_scroll(column, grab, pos, bounds);
                return;
            }

            self.state.pressed = Some(Pressed::TimePress {
                column,
                start_y: pos.y,
                start_scroll: scroll,
            });
            return;
        }
    }

    /// Selects the time item under `pos`, if any (on release of an
    /// unmoved press in `column`).
    fn time_release(
        &mut self,
        column: TimeColumn,
        pos: Point,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        let regions = time_regions(self.time_origin(bounds));
        let col = match column {
            TimeColumn::Hour => regions.hour_col,
            TimeColumn::Minute => regions.minute_col,
        };

        if !col.contains(pos) {
            return;
        }

        let count = self.item_count(column);
        let index = ((pos.y - col.y + self.state.scroll(column)) / TIME_ITEM_H).floor() as usize;
        if index < count {
            self.pick_time(column, index, shell);
        }
    }

    /// Updates a column's scroll offset from a scrollbar drag.
    fn drag_scroll(&mut self, column: TimeColumn, grab: f32, pos: Point, bounds: Rectangle) {
        let regions = time_regions(self.time_origin(bounds));
        let col = match column {
            TimeColumn::Hour => regions.hour_col,
            TimeColumn::Minute => regions.minute_col,
        };

        let count = self.item_count(column);
        let Some(sb) = scrollbar(col, count, self.state.scroll(column)) else {
            return;
        };

        let span = sb.track.height - sb.thumb.height;
        if span <= 0.0 {
            return;
        }

        let fraction = ((pos.y - grab - sb.track.y) / span).clamp(0.0, 1.0);
        self.set_scroll(column, fraction * max_scroll(count));
    }
}

impl<'a, 'b, Message> overlay::Overlay<Message, Theme, iced::Renderer>
    for PickerOverlay<'a, 'b, Message>
where
    Message: Clone + 'a,
{
    fn layout(&mut self, _renderer: &iced::Renderer, viewport: Size) -> layout::Node {
        let size = popup_size(self.mode);

        // Position below the field, or above if there is no room.
        let mut x = self.field_bounds.x;
        let mut y = self.field_bounds.y + self.field_bounds.height + POPUP_OFFSET;

        if y + size.height > viewport.height {
            y = self.field_bounds.y - size.height - POPUP_OFFSET;
        }
        if x + size.width > viewport.width {
            x = viewport.width - size.width;
        }
        x = x.max(0.0);
        y = y.max(0.0);

        layout::Node::new(size).move_to(Point::new(x, y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position() else {
                    self.close(shell);
                    return;
                };

                if !bounds.contains(pos) {
                    // Click outside the popup: dismiss.
                    self.close(shell);

                    let field_text_area = Rectangle {
                        width: self.field_bounds.width - TRIGGER_WIDTH,
                        ..self.field_bounds
                    };
                    // Let clicks on the field's text area pass through
                    // so the press also focuses the inner text input.
                    // Everything else (including the trigger, so it
                    // acts as a toggle) is swallowed.
                    if !field_text_area.contains(pos) {
                        shell.capture_event();
                    }
                    return;
                }

                // Arm a press; the action fires on release.
                if self.mode.has_calendar()
                    && let Some(target) = self.calendar_target(pos, bounds)
                {
                    self.state.pressed = Some(Pressed::Target(target));
                } else if self.mode.has_time() {
                    self.time_press(pos, bounds);
                }

                // Clicks inside the popup never reach the widgets
                // underneath.
                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let Some(pressed) = self.state.pressed.take() else {
                    return;
                };

                match pressed {
                    Pressed::Target(target) => {
                        // Activate the target under the release
                        // position: grid cells accept a press dragged
                        // in from another cell of the same kind, while
                        // header buttons require press and release on
                        // the same control.
                        if let Some(released) = cursor
                            .position()
                            .and_then(|pos| self.calendar_target(pos, bounds))
                            && release_activates(target, released)
                        {
                            self.activate_target(released, shell);
                        }
                    }
                    Pressed::TimePress { column, .. } => {
                        // An unmoved press is a click: select the item
                        // under the cursor.
                        if let Some(pos) = cursor.position() {
                            self.time_release(column, pos, bounds, shell);
                        }
                    }
                    // Ending a drag selects nothing.
                    Pressed::Scrollbar(..) | Pressed::TimeDrag { .. } => {}
                }

                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position() {
                    match self.state.pressed {
                        Some(Pressed::Scrollbar(column, grab)) => {
                            self.drag_scroll(column, grab, pos, bounds);
                        }
                        Some(Pressed::TimePress {
                            column,
                            start_y,
                            start_scroll,
                        }) if (pos.y - start_y).abs() > DRAG_THRESHOLD => {
                            // The press moved: it is a content drag,
                            // not a click.
                            self.state.pressed = Some(Pressed::TimeDrag {
                                column,
                                start_y,
                                start_scroll,
                            });
                            self.set_scroll(column, start_scroll - (pos.y - start_y));
                        }
                        Some(Pressed::TimeDrag {
                            column,
                            start_y,
                            start_scroll,
                        }) => {
                            // The content follows the cursor.
                            self.set_scroll(column, start_scroll - (pos.y - start_y));
                        }
                        _ => {}
                    }
                }
                // Keep hover highlights fresh.
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if self.mode.has_time()
                    && let Some(pos) = cursor.position()
                {
                    let regions = time_regions(self.time_origin(bounds));
                    for (column, col) in [
                        (TimeColumn::Hour, regions.hour_col),
                        (TimeColumn::Minute, regions.minute_col),
                    ] {
                        if col.contains(pos) {
                            let dy = match delta {
                                mouse::ScrollDelta::Lines { y, .. } => y * TIME_ITEM_H,
                                mouse::ScrollDelta::Pixels { y, .. } => *y,
                            };
                            self.set_scroll(column, self.state.scroll(column) - dy);
                            shell.request_redraw();
                            shell.capture_event();
                            return;
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                self.close(shell);
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let style = (self.class)(theme);
        let bounds = layout.bounds();
        let text_size = theme.text(0.8125);
        let label_size = theme.text(0.6875);

        // Popup background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.popup_border,
                ..renderer::Quad::default()
            },
            style.popup_background,
        );

        if self.mode.has_calendar() {
            self.draw_calendar(renderer, &style, bounds, cursor, text_size, label_size);
        }

        if self.mode.has_time() {
            self.draw_time(renderer, &style, bounds, cursor, text_size, label_size);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.state.is_time_dragging() {
            return mouse::Interaction::Grabbing;
        }

        let bounds = layout.bounds();
        let Some(pos) = cursor.position_over(bounds) else {
            return mouse::Interaction::default();
        };

        if self.mode.has_calendar() {
            let origin = self.content_origin(bounds);
            let regions = calendar_regions(origin);
            let body = calendar_body(origin);

            if regions.prev.contains(pos) || regions.next.contains(pos) {
                return mouse::Interaction::Pointer;
            }

            match self.state.view {
                CalendarView::Days => {
                    if regions.title.contains(pos) {
                        return mouse::Interaction::Pointer;
                    }
                    if let Some(index) =
                        grid::cell_at(pos, regions.grid, grid::GRID_COLS, grid::GRID_ROWS)
                    {
                        let date =
                            grid::grid_dates(self.state.view_year, self.state.view_month)[index];
                        if self.date_enabled(date) {
                            return mouse::Interaction::Pointer;
                        }
                    }
                }
                CalendarView::Months => {
                    if regions.title.contains(pos) {
                        return mouse::Interaction::Pointer;
                    }
                    if let Some(index) =
                        grid::cell_at(pos, body, grid::MONTH_COLS, grid::MONTH_ROWS)
                        && self.month_enabled(self.state.view_year, index as u32 + 1)
                    {
                        return mouse::Interaction::Pointer;
                    }
                }
                CalendarView::Years => {
                    if let Some(index) = grid::cell_at(pos, body, grid::YEAR_COLS, grid::YEAR_ROWS)
                        && self.year_enabled(
                            grid::year_page_start(self.state.view_year) + index as i32,
                        )
                    {
                        return mouse::Interaction::Pointer;
                    }
                }
            }
        }

        if self.mode.has_time() {
            let regions = time_regions(self.time_origin(bounds));
            for (column, col) in [
                (TimeColumn::Hour, regions.hour_col),
                (TimeColumn::Minute, regions.minute_col),
            ] {
                if !col.contains(pos) {
                    continue;
                }
                let count = self.item_count(column);
                if let Some(sb) = scrollbar(col, count, self.state.scroll(column))
                    && sb.track.contains(pos)
                {
                    return mouse::Interaction::Grab;
                }
                return mouse::Interaction::Pointer;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a, 'b, Message> PickerOverlay<'a, 'b, Message> {
    fn draw_calendar(
        &self,
        renderer: &mut iced::Renderer,
        style: &Style,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        text_size: iced::Pixels,
        label_size: iced::Pixels,
    ) {
        let origin = self.content_origin(bounds);
        let regions = calendar_regions(origin);
        let body = calendar_body(origin);

        let pos = cursor.position();
        let hover_segment = |renderer: &mut iced::Renderer, segment: Rectangle| {
            if pos.is_some_and(|pos| segment.contains(pos)) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: inset(segment, 2.0),
                        border: Border {
                            radius: style.cell_radius.into(),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    style.cell_hover_background,
                );
            }
        };

        // Header chevrons (month, year, or year-page navigation
        // depending on the view).
        hover_segment(renderer, regions.prev);
        hover_segment(renderer, regions.next);
        fill_label(renderer, "‹", regions.prev, text_size, style.title_color);
        fill_label(renderer, "›", regions.next, text_size, style.title_color);

        match self.state.view {
            CalendarView::Days => {
                // Title: clickable month and year segments.
                let (month_seg, year_seg) = title_segments(regions.title);
                hover_segment(renderer, month_seg);
                hover_segment(renderer, year_seg);
                fill_label(
                    renderer,
                    &grid::month_name(self.state.view_month),
                    month_seg,
                    text_size,
                    style.title_color,
                );
                fill_label(
                    renderer,
                    &self.state.view_year.to_string(),
                    year_seg,
                    text_size,
                    style.title_color,
                );

                self.draw_days(renderer, style, &regions, cursor, text_size, label_size);
            }
            CalendarView::Months => {
                // Title: the view year, clickable to the year list.
                hover_segment(renderer, regions.title);
                fill_label(
                    renderer,
                    &self.state.view_year.to_string(),
                    regions.title,
                    text_size,
                    style.title_color,
                );

                let hovered = pos
                    .and_then(|pos| grid::cell_at(pos, body, grid::MONTH_COLS, grid::MONTH_ROWS));
                for (index, label) in grid::MONTH_LABELS.iter().enumerate() {
                    let rect = grid::cell_rect(body, grid::MONTH_COLS, grid::MONTH_ROWS, index);
                    let month = index as u32 + 1;
                    draw_pick_cell(
                        renderer,
                        style,
                        rect,
                        label,
                        self.state.edit.date().year() == self.state.view_year
                            && self.state.edit.date().month() == month,
                        hovered == Some(index),
                        self.month_enabled(self.state.view_year, month),
                        text_size,
                    );
                }
            }
            CalendarView::Years => {
                fill_label(
                    renderer,
                    &grid::year_page_title(self.state.view_year),
                    regions.title,
                    text_size,
                    style.title_color,
                );

                let start = grid::year_page_start(self.state.view_year);
                let hovered =
                    pos.and_then(|pos| grid::cell_at(pos, body, grid::YEAR_COLS, grid::YEAR_ROWS));
                for index in 0..(grid::YEAR_COLS * grid::YEAR_ROWS) {
                    let rect = grid::cell_rect(body, grid::YEAR_COLS, grid::YEAR_ROWS, index);
                    let year = start + index as i32;
                    draw_pick_cell(
                        renderer,
                        style,
                        rect,
                        &year.to_string(),
                        self.state.edit.date().year() == year,
                        hovered == Some(index),
                        self.year_enabled(year),
                        text_size,
                    );
                }
            }
        }
    }

    /// Draws the weekday labels and the day grid of the day view.
    fn draw_days(
        &self,
        renderer: &mut iced::Renderer,
        style: &Style,
        regions: &CalendarRegions,
        cursor: mouse::Cursor,
        text_size: iced::Pixels,
        label_size: iced::Pixels,
    ) {
        // Weekday labels
        for (i, label) in grid::WEEKDAY_LABELS.iter().enumerate() {
            let rect = grid::cell_rect(regions.weekdays, grid::GRID_COLS, 1, i);
            fill_label(renderer, label, rect, label_size, style.label_color);
        }

        // Day cells
        let dates = grid::grid_dates(self.state.view_year, self.state.view_month);
        let selected = self.state.edit.date();
        let hovered = cursor
            .position()
            .and_then(|pos| grid::cell_at(pos, regions.grid, grid::GRID_COLS, grid::GRID_ROWS));

        for (i, date) in dates.iter().enumerate() {
            let rect = grid::cell_rect(regions.grid, grid::GRID_COLS, grid::GRID_ROWS, i);
            let highlight = inset(rect, 1.0);
            let enabled = self.date_enabled(*date);
            let in_view_month =
                date.year() == self.state.view_year && date.month() == self.state.view_month;
            let is_selected = *date == selected;

            let text_color = if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: highlight,
                        border: Border {
                            radius: style.cell_radius.into(),
                            ..Border::default()
                        },
                        ..renderer::Quad::default()
                    },
                    style.selected_background,
                );
                style.selected_text_color
            } else {
                if hovered == Some(i) && enabled {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: highlight,
                            border: Border {
                                radius: style.cell_radius.into(),
                                ..Border::default()
                            },
                            ..renderer::Quad::default()
                        },
                        style.cell_hover_background,
                    );
                }
                // Outline today's date (selected fill takes precedence).
                if *date == self.state.today {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: highlight,
                            border: Border {
                                color: style.today_border_color,
                                width: 1.0,
                                radius: style.cell_radius.into(),
                            },
                            ..renderer::Quad::default()
                        },
                        Background::Color(Color::TRANSPARENT),
                    );
                }
                if enabled && in_view_month {
                    style.cell_text_color
                } else {
                    style.cell_muted_color
                }
            };

            fill_label(
                renderer,
                &date.day().to_string(),
                rect,
                text_size,
                text_color,
            );
        }
    }

    fn draw_time(
        &self,
        renderer: &mut iced::Renderer,
        style: &Style,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        text_size: iced::Pixels,
        label_size: iced::Pixels,
    ) {
        let regions = time_regions(self.time_origin(bounds));

        fill_label(
            renderer,
            "Hour",
            regions.hour_label,
            label_size,
            style.label_color,
        );
        fill_label(
            renderer,
            "Minute",
            regions.minute_label,
            label_size,
            style.label_color,
        );

        for (column, col) in [
            (TimeColumn::Hour, regions.hour_col),
            (TimeColumn::Minute, regions.minute_col),
        ] {
            self.draw_time_column(renderer, style, column, col, cursor, text_size);
        }
    }

    /// Draws one scrollable time column: its visible items, clipped to
    /// the column bounds, plus the scrollbar thumb.
    fn draw_time_column(
        &self,
        renderer: &mut iced::Renderer,
        style: &Style,
        column: TimeColumn,
        col: Rectangle,
        cursor: mouse::Cursor,
        text_size: iced::Pixels,
    ) {
        let count = self.item_count(column);
        let scroll = self.state.scroll(column);
        let selected = self.selected_index(column);
        let scrollbar = scrollbar(col, count, scroll);

        // The hovered item, ignoring the scrollbar area and any
        // in-progress drag.
        let hovered = cursor.position().and_then(|pos| {
            if !col.contains(pos) || self.state.is_time_dragging() {
                return None;
            }
            if let Some(sb) = &scrollbar
                && sb.track.contains(pos)
            {
                return None;
            }
            let index = ((pos.y - col.y + scroll) / TIME_ITEM_H).floor() as usize;
            (index < count).then_some(index)
        });

        // Reserve a lane for the scrollbar so items don't run under
        // the thumb.
        let item_w = if scrollbar.is_some() {
            col.width - (SCROLLBAR_W + SCROLLBAR_MARGIN * 2.0)
        } else {
            col.width
        };

        renderer.with_layer(col, |renderer| {
            let first = (scroll / TIME_ITEM_H).floor() as usize;
            let last = (((scroll + col.height) / TIME_ITEM_H).ceil() as usize).min(count);

            for index in first..last {
                let rect = Rectangle::new(
                    Point::new(col.x, col.y + index as f32 * TIME_ITEM_H - scroll),
                    Size::new(item_w, TIME_ITEM_H),
                );
                let value = match column {
                    TimeColumn::Hour => index as u32,
                    TimeColumn::Minute => index as u32 * self.minute_step,
                };
                draw_pick_cell(
                    renderer,
                    style,
                    rect,
                    &format!("{value:02}"),
                    selected == Some(index),
                    hovered == Some(index),
                    true,
                    text_size,
                );
            }
        });

        if let Some(sb) = scrollbar {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: sb.thumb,
                    border: Border {
                        radius: (SCROLLBAR_W / 2.0).into(),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(style.scrollbar_thumb),
            );
        }
    }
}

// -- Drawing helpers --

/// Draws a selectable cell (month, year, hour, or minute): selected
/// fill, hover background, and range-aware text color.
#[allow(clippy::too_many_arguments)]
fn draw_pick_cell(
    renderer: &mut iced::Renderer,
    style: &Style,
    rect: Rectangle,
    label: &str,
    is_selected: bool,
    is_hovered: bool,
    is_enabled: bool,
    text_size: iced::Pixels,
) {
    let highlight = inset(rect, 1.0);
    let text_color = if is_selected {
        renderer.fill_quad(
            renderer::Quad {
                bounds: highlight,
                border: Border {
                    radius: style.cell_radius.into(),
                    ..Border::default()
                },
                ..renderer::Quad::default()
            },
            style.selected_background,
        );
        style.selected_text_color
    } else {
        if is_hovered && is_enabled {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: highlight,
                    border: Border {
                        radius: style.cell_radius.into(),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                style.cell_hover_background,
            );
        }
        if is_enabled {
            style.cell_text_color
        } else {
            style.cell_muted_color
        }
    };

    fill_label(renderer, label, rect, text_size, text_color);
}

/// Draws `content` centered inside `bounds`.
fn fill_label(
    renderer: &mut iced::Renderer,
    content: &str,
    bounds: Rectangle,
    size: iced::Pixels,
    color: Color,
) {
    renderer.fill_text(
        iced::advanced::Text {
            content: content.to_string(),
            bounds: bounds.size(),
            size,
            line_height: text::LineHeight::Relative(1.0),
            font: renderer.default_font(),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        },
        Point::new(bounds.center_x(), bounds.center_y()),
        color,
        bounds,
    );
}

/// Draws a small calendar glyph centered in `bounds` using quads.
fn draw_calendar_icon(renderer: &mut iced::Renderer, bounds: Rectangle, color: Color) {
    let size = 14.0;
    let origin = Point::new(
        bounds.center_x() - size / 2.0,
        bounds.center_y() - size / 2.0,
    );

    // Body outline (slightly shorter than the full glyph, leaving room
    // for the binding pins above).
    let body = Rectangle::new(
        Point::new(origin.x, origin.y + 2.0),
        Size::new(size, size - 2.0),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: body,
            border: Border {
                color,
                width: 1.5,
                radius: 2.0.into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );

    // Header bar.
    let header = Rectangle::new(
        Point::new(origin.x + 1.5, origin.y + 3.5),
        Size::new(size - 3.0, 3.0),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: header,
            ..renderer::Quad::default()
        },
        Background::Color(color),
    );

    // Binding pins.
    for x in [origin.x + 3.0, origin.x + size - 4.5] {
        let pin = Rectangle::new(Point::new(x, origin.y), Size::new(1.5, 4.0));
        renderer.fill_quad(
            renderer::Quad {
                bounds: pin,
                ..renderer::Quad::default()
            },
            Background::Color(color),
        );
    }
}

/// Draws a small clock glyph centered in `bounds` using quads.
fn draw_clock_icon(renderer: &mut iced::Renderer, bounds: Rectangle, color: Color) {
    let size = 14.0;
    let center = Point::new(bounds.center_x(), bounds.center_y());
    let face = Rectangle::new(
        Point::new(center.x - size / 2.0, center.y - size / 2.0),
        Size::new(size, size),
    );

    // Face outline (a fully-rounded quad).
    renderer.fill_quad(
        renderer::Quad {
            bounds: face,
            border: Border {
                color,
                width: 1.5,
                radius: (size / 2.0).into(),
            },
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );

    // Minute hand (pointing up).
    let minute_hand = Rectangle::new(
        Point::new(center.x - 0.75, center.y - 4.0),
        Size::new(1.5, 4.5),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: minute_hand,
            ..renderer::Quad::default()
        },
        Background::Color(color),
    );

    // Hour hand (pointing right).
    let hour_hand = Rectangle::new(
        Point::new(center.x - 0.75, center.y - 0.75),
        Size::new(3.75, 1.5),
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: hour_hand,
            ..renderer::Quad::default()
        },
        Background::Color(color),
    );
}

impl<'a, Message> From<Picker<'a, Message>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    fn from(picker: Picker<'a, Message>) -> Self {
        Element::new(picker)
    }
}
