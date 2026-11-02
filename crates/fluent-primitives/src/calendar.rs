use fluent_core::ThemeProvider as _;

type CalendarChange = std::rc::Rc<dyn Fn(CalendarDate, &mut App)>;
use gpui::{
    div, prelude::*, px, App, ClickEvent, Context, ElementId, IntoElement, KeyDownEvent, Render,
    Role, SharedString, Window,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A Fluent primitive for representing calendar date in an interface.
pub struct CalendarDate {
    /// The year value.
    pub year: i32,
    /// The month value.
    pub month: u8,
    /// The day value.
    pub day: u8,
}
impl CalendarDate {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        let max = days_in_month(year, month)?;
        (day > 0 && day <= max).then_some(Self { year, month, day })
    }
}

#[derive(Clone)]
/// A Fluent primitive for representing calendar in an interface.
pub struct Calendar {
    id: ElementId,
    value: CalendarDate,
    month: CalendarDate,
    first_day_of_week: u8,
    min: Option<CalendarDate>,
    max: Option<CalendarDate>,
    label: SharedString,
    on_change: Option<CalendarChange>,
}
impl Calendar {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>, value: CalendarDate) -> Self {
        Self {
            id: id.into(),
            value,
            month: CalendarDate { day: 1, ..value },
            first_day_of_week: 0,
            min: None,
            max: None,
            label: "Calendar".into(),
            on_change: None,
        }
    }
    /// Returns the current value.
    pub fn value(mut self, value: CalendarDate) -> Self {
        self.value = value;
        self.month = CalendarDate { day: 1, ..value };
        self
    }
    /// Sets the first day of week configuration for this control.
    pub fn first_day_of_week(mut self, day: u8) -> Self {
        self.first_day_of_week = day % 7;
        self
    }
    /// Sets the locale first day of week configuration for this control.
    pub fn locale_first_day_of_week(self, day: u8) -> Self {
        self.first_day_of_week(day)
    }
    /// Sets the min configuration for this control.
    pub fn min(mut self, date: CalendarDate) -> Self {
        self.min = Some(date);
        self
    }
    /// Sets the max configuration for this control.
    pub fn max(mut self, date: CalendarDate) -> Self {
        self.max = Some(date);
        self
    }
    /// Returns the current label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
    /// Registers a handler for change events.
    pub fn on_change(mut self, f: impl Fn(CalendarDate, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(f));
        self
    }
    /// Move the active date by a keyboard grid step. Returns `None` when the move is outside bounds.
    pub fn keyboard_move(&self, key: &str) -> Option<CalendarDate> {
        let delta = match key {
            "left" => -1,
            "right" => 1,
            "up" => -7,
            "down" => 7,
            _ => return None,
        };
        let candidate = add_days(self.value, delta);
        self.allowed(candidate).then_some(candidate)
    }

    /// Returns the current selected.
    pub fn selected(&self) -> CalendarDate {
        self.value
    }
    fn allowed(&self, date: CalendarDate) -> bool {
        self.min.is_none_or(|d| date >= d) && self.max.is_none_or(|d| date <= d)
    }
}
impl Render for Calendar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let month = self.month;
        let first = self.first_day_of_week;
        let selected = self.value;
        let change = self.on_change.clone();
        let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let mut weekdays = div().flex().gap_1();
        for i in 0..7 {
            weekdays = weekdays.child(
                div()
                    .w(px(32.))
                    .text_center()
                    .text_size(px(11.))
                    .text_color(colors.on_subtle)
                    .child(names[((i + first) % 7) as usize]),
            );
        }
        let offset = (weekday(month.year, month.month) + 7 - first) % 7;
        let count = days_in_month(month.year, month.month).unwrap_or(28);
        let mut grid = div().flex().flex_wrap().gap_1();
        for slot in 0..(offset + count) {
            if slot < offset {
                grid = grid.child(div().size(px(32.)));
                continue;
            }
            let day = slot - offset + 1;
            let date = CalendarDate { day, ..month };
            let enabled = self.allowed(date);
            let is_selected = date == selected;
            let handler = change.clone();
            let mut cell = div()
                .id(("calendar-day", day as u64))
                .size(px(32.))
                .flex()
                .items_center()
                .justify_center()
                .role(Role::GridCell);
            if enabled {
                cell = cell
                    .cursor_pointer()
                    .when(is_selected, |d| {
                        d.bg(colors.accent).text_color(colors.on_accent)
                    })
                    .on_click(move |_: &ClickEvent, _, app| {
                        if let Some(f) = &handler {
                            f(date, app);
                        }
                    });
            } else {
                cell = cell.text_color(colors.on_neutral_disabled);
            }
            grid = grid.child(cell.child(day.to_string()));
        }
        div()
            .id(self.id.clone())
            .role(Role::Grid)
            .flex()
            .flex_col()
            .gap(px(spacing.sm))
            .child(
                div()
                    .text_size(px(14.))
                    .child(format!("{} / {}", month.month, month.year)),
            )
            .child(weekdays)
            .child(grid)
            .on_key_down(move |ev: &KeyDownEvent, _, app| {
                if matches!(ev.keystroke.key.as_str(), "left" | "right" | "up" | "down") {
                    app.stop_propagation();
                }
            })
    }
}
fn add_days(mut date: CalendarDate, delta: i32) -> CalendarDate {
    let step = if delta < 0 { -1 } else { 1 };
    for _ in 0..delta.unsigned_abs() {
        if step > 0 {
            if date.day < days_in_month(date.year, date.month).unwrap_or(28) {
                date.day += 1;
            } else if date.month == 12 {
                date = CalendarDate {
                    year: date.year + 1,
                    month: 1,
                    day: 1,
                };
            } else {
                date = CalendarDate {
                    year: date.year,
                    month: date.month + 1,
                    day: 1,
                };
            }
        } else if date.day > 1 {
            date.day -= 1;
        } else if date.month == 1 {
            date = CalendarDate {
                year: date.year - 1,
                month: 12,
                day: 31,
            };
        } else {
            let month = date.month - 1;
            date = CalendarDate {
                year: date.year,
                month,
                day: days_in_month(date.year, month).unwrap_or(28),
            };
        }
    }
    date
}
fn days_in_month(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => Some(29),
        2 => Some(28),
        _ => None,
    }
}
fn weekday(year: i32, month: u8) -> u8 {
    let (y, m) = if month < 3 {
        (year - 1, month as i32 + 12)
    } else {
        (year, month as i32)
    };
    ((y + y / 4 - y / 100 + y / 400 + (13 * m + 8) / 5 + 1) % 7) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calendar_date_rejects_invalid_days() {
        assert!(CalendarDate::new(2024, 2, 29).is_some());
        assert!(CalendarDate::new(2023, 2, 29).is_none());
    }
    #[test]
    fn calendar_week_starts_with_locale_offset() {
        assert_eq!(weekday(2024, 1), 1);
    }
}
