//! Pure calendar/time grid math shared by the date/time picker
//! overlays.
//!
//! All functions here are side-effect free and unit-tested at the
//! bottom of this file.

use chrono::{Datelike, Days, NaiveDate};
use iced::{Point, Rectangle};

/// Monday-first weekday header labels.
pub(crate) const WEEKDAY_LABELS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// Number of columns in the calendar day grid.
pub(crate) const GRID_COLS: usize = 7;

/// Number of rows in the calendar day grid.
pub(crate) const GRID_ROWS: usize = 6;

/// Abbreviated month labels shown in the month list view.
pub(crate) const MONTH_LABELS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Number of columns in the month list grid.
pub(crate) const MONTH_COLS: usize = 3;

/// Number of rows in the month list grid.
pub(crate) const MONTH_ROWS: usize = 4;

/// Number of columns in the year list grid.
pub(crate) const YEAR_COLS: usize = 4;

/// Number of rows in the year list grid.
pub(crate) const YEAR_ROWS: usize = 5;

/// Number of years shown per page of the year list view.
pub(crate) const YEARS_PER_PAGE: i32 = (YEAR_COLS * YEAR_ROWS) as i32;

/// Returns the first year of the year-list page containing `year`:
/// `year` floored to a multiple of [`YEARS_PER_PAGE`].
pub(crate) fn year_page_start(year: i32) -> i32 {
    year - year.rem_euclid(YEARS_PER_PAGE)
}

/// Formats the year list header title, e.g. `"2020 – 2039"`.
pub(crate) fn year_page_title(year: i32) -> String {
    let start = year_page_start(year);
    format!("{start} – {}", start + YEARS_PER_PAGE - 1)
}

/// Returns the date shown in the top-left cell of the day grid for
/// the given view month: the Monday on or before the 1st.
pub(crate) fn first_grid_day(year: i32, month: u32) -> NaiveDate {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid year/month");
    let offset = u64::from(first.weekday().num_days_from_monday());
    first - Days::new(offset)
}

/// Returns all 42 dates (6 rows × 7 columns, Monday-first) shown in
/// the day grid for the given view month.
pub(crate) fn grid_dates(year: i32, month: u32) -> [NaiveDate; GRID_COLS * GRID_ROWS] {
    let start = first_grid_day(year, month);
    std::array::from_fn(|i| start + Days::new(i as u64))
}

/// Returns the view month preceding `(year, month)`.
pub(crate) fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

/// Returns the view month following `(year, month)`.
pub(crate) fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

/// Returns the full English month name, e.g. `"June"`.
pub(crate) fn month_name(month: u32) -> String {
    NaiveDate::from_ymd_opt(2000, month, 1)
        .expect("valid month")
        .format("%B")
        .to_string()
}

/// Returns the index of the cell under `pos` in a uniform
/// `cols` × `rows` grid covering `grid`, or `None` if `pos` is
/// outside the grid.
pub(crate) fn cell_at(pos: Point, grid: Rectangle, cols: usize, rows: usize) -> Option<usize> {
    if !grid.contains(pos) {
        return None;
    }
    let col = (((pos.x - grid.x) / (grid.width / cols as f32)) as usize).min(cols - 1);
    let row = (((pos.y - grid.y) / (grid.height / rows as f32)) as usize).min(rows - 1);
    Some(row * cols + col)
}

/// Returns the bounds of cell `index` in a uniform `cols` × `rows`
/// grid covering `grid`.
pub(crate) fn cell_rect(grid: Rectangle, cols: usize, rows: usize, index: usize) -> Rectangle {
    let cell_w = grid.width / cols as f32;
    let cell_h = grid.height / rows as f32;
    let col = index % cols;
    let row = index / cols;
    Rectangle {
        x: grid.x + col as f32 * cell_w,
        y: grid.y + row as f32 * cell_h,
        width: cell_w,
        height: cell_h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn first_grid_day_when_month_starts_on_monday() {
        // June 2026 starts on a Monday.
        assert_eq!(first_grid_day(2026, 6), date(2026, 6, 1));
    }

    #[test]
    fn first_grid_day_when_month_starts_midweek() {
        // May 2026 starts on a Friday → grid starts Monday April 27.
        assert_eq!(first_grid_day(2026, 5), date(2026, 4, 27));
    }

    #[test]
    fn first_grid_day_when_month_starts_on_sunday() {
        // March 2026 starts on a Sunday → grid starts Monday Feb 23.
        assert_eq!(first_grid_day(2026, 3), date(2026, 2, 23));
    }

    #[test]
    fn grid_dates_cover_leap_february() {
        // February 2024 (leap year) starts on a Thursday.
        let dates = grid_dates(2024, 2);
        assert_eq!(dates[0], date(2024, 1, 29));
        assert!(dates.contains(&date(2024, 2, 29)));
        assert_eq!(dates[41], date(2024, 3, 10));
    }

    #[test]
    fn month_navigation_wraps_year_boundaries() {
        assert_eq!(prev_month(2026, 1), (2025, 12));
        assert_eq!(next_month(2025, 12), (2026, 1));
        assert_eq!(prev_month(2026, 7), (2026, 6));
        assert_eq!(next_month(2026, 7), (2026, 8));
    }

    #[test]
    fn month_name_formats_english() {
        assert_eq!(month_name(6), "June");
        assert_eq!(month_name(2), "February");
        assert_eq!(MONTH_LABELS[5], "Jun");
    }

    #[test]
    fn year_page_start_floors_to_page_multiples() {
        assert_eq!(year_page_start(2026), 2020);
        assert_eq!(year_page_start(2020), 2020);
        assert_eq!(year_page_start(2039), 2020);
        assert_eq!(year_page_start(2040), 2040);
        assert_eq!(year_page_start(0), 0);
        assert_eq!(year_page_start(-1), -20);
        assert_eq!(year_page_start(-20), -20);
    }

    #[test]
    fn year_page_title_formats_range() {
        assert_eq!(year_page_title(2026), "2020 – 2039");
        assert_eq!(year_page_title(1999), "1980 – 1999");
    }

    #[test]
    fn cell_hit_testing_round_trips() {
        let grid = Rectangle::new(Point::new(10.0, 20.0), Size::new(210.0, 156.0));
        for index in [0, 6, 20, 35, 41] {
            let rect = cell_rect(grid, GRID_COLS, GRID_ROWS, index);
            let center = Point::new(rect.center_x(), rect.center_y());
            assert_eq!(cell_at(center, grid, GRID_COLS, GRID_ROWS), Some(index));
        }
        assert_eq!(
            cell_at(Point::new(0.0, 0.0), grid, GRID_COLS, GRID_ROWS),
            None
        );
    }
}
