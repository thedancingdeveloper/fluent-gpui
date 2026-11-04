use fluent_core::TextRampExt;
use std::sync::{Arc, Mutex};

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, svg, Anchor, App, Bounds, ClickEvent,
    Context, CursorStyle, FocusHandle, Focusable, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, Pixels, Point, Render, SharedString, Window,
};

const MIN_YEAR: i32 = 1;
const MAX_YEAR: i32 = 9999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A Fluent primitive for representing date time value in an interface.
pub struct DateTimeValue {
    /// The year value.
    pub year: i32,
    /// The month value.
    pub month: u8,
    /// The day value.
    pub day: u8,
    /// The hour value.
    pub hour: u8,
    /// The minute value.
    pub minute: u8,
}

impl DateTimeValue {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> Option<Self> {
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) || hour > 23 || minute > 59 {
            return None;
        }
        let max_day = Self::days_in_month(year, month)?;
        if day == 0 || day > max_day {
            return None;
        }
        Some(Self {
            year,
            month,
            day,
            hour,
            minute,
        })
    }

    /// Parses segmented input as a local date and time.
    pub fn parse_local(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        let split = trimmed.find('T').or_else(|| trimmed.find(' '))?;
        let (date, time_with_sep) = trimmed.split_at(split);
        let time = &time_with_sep[1..];

        let mut date_parts = date.split('-');
        let year = date_parts.next()?.parse().ok()?;
        let month = date_parts.next()?.parse().ok()?;
        let day = date_parts.next()?.parse().ok()?;
        if date_parts.next().is_some() {
            return None;
        }

        let mut time_parts = time.split(':');
        let hour = time_parts.next()?.parse().ok()?;
        let minute = time_parts.next()?.parse().ok()?;
        if time_parts.next().is_some() {
            return None;
        }

        Self::new(year, month, day, hour, minute)
    }

    /// Formats the value for use in a local date-time input.
    pub fn to_local_input_value(self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }

    /// Formats the value for display to the user.
    pub fn display_value(self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }

    /// Returns whether leap year.
    pub fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Returns the number of days in this value’s month.
    pub fn days_in_month(year: i32, month: u8) -> Option<u8> {
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return None;
        }
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
            4 | 6 | 9 | 11 => Some(30),
            2 if Self::is_leap_year(year) => Some(29),
            2 => Some(28),
            _ => None,
        }
    }

    /// Returns a copy adjusted by the requested segment and amount.
    pub fn adjust(self, segment: DateTimeSegment, delta: i32) -> Self {
        match segment {
            DateTimeSegment::Year => self.add_years(delta),
            DateTimeSegment::Month => self.add_months(delta),
            DateTimeSegment::Day => self.add_days(delta),
            DateTimeSegment::Hour => self.add_hours(delta),
            DateTimeSegment::Minute => self.add_minutes(delta),
        }
    }

    fn add_years(self, delta: i32) -> Self {
        let year = self.year.saturating_add(delta).clamp(MIN_YEAR, MAX_YEAR);
        self.with_year_month(year, self.month)
    }

    fn add_months(self, delta: i32) -> Self {
        let month_index = (self.year - MIN_YEAR) * 12 + i32::from(self.month - 1);
        let max_index = (MAX_YEAR - MIN_YEAR) * 12 + 11;
        let next = month_index.saturating_add(delta).clamp(0, max_index);
        let year = MIN_YEAR + next / 12;
        let month = (next % 12 + 1) as u8;
        self.with_year_month(year, month)
    }

    fn add_days(self, delta: i32) -> Self {
        let mut next = self;
        if delta > 0 {
            for _ in 0..delta {
                if !next.step_next_day() {
                    break;
                }
            }
        } else {
            for _ in delta..0 {
                if !next.step_previous_day() {
                    break;
                }
            }
        }
        next
    }

    fn add_hours(self, delta: i32) -> Self {
        let total = i32::from(self.hour).saturating_add(delta);
        let day_delta = total.div_euclid(24);
        let hour = total.rem_euclid(24) as u8;
        Self {
            hour,
            ..self.add_days(day_delta)
        }
    }

    fn add_minutes(self, delta: i32) -> Self {
        let total = i32::from(self.hour) * 60 + i32::from(self.minute);
        let total = total.saturating_add(delta);
        let day_delta = total.div_euclid(24 * 60);
        let minute_of_day = total.rem_euclid(24 * 60);
        Self {
            hour: (minute_of_day / 60) as u8,
            minute: (minute_of_day % 60) as u8,
            ..self.add_days(day_delta)
        }
    }

    fn with_year_month(self, year: i32, month: u8) -> Self {
        let max_day = Self::days_in_month(year, month).unwrap_or(31);
        Self {
            year,
            month,
            day: self.day.min(max_day),
            ..self
        }
    }

    fn step_next_day(&mut self) -> bool {
        let max_day = Self::days_in_month(self.year, self.month).unwrap_or(31);
        if self.day < max_day {
            self.day += 1;
            return true;
        }
        if self.month < 12 {
            self.month += 1;
            self.day = 1;
            return true;
        }
        if self.year < MAX_YEAR {
            self.year += 1;
            self.month = 1;
            self.day = 1;
            return true;
        }
        false
    }

    fn step_previous_day(&mut self) -> bool {
        if self.day > 1 {
            self.day -= 1;
            return true;
        }
        if self.month > 1 {
            self.month -= 1;
            self.day = Self::days_in_month(self.year, self.month).unwrap_or(31);
            return true;
        }
        if self.year > MIN_YEAR {
            self.year -= 1;
            self.month = 12;
            self.day = 31;
            return true;
        }
        false
    }
}

/// Order of the date fields in a locale's display and entry format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateOrder {
    /// The year month day option.
    YearMonthDay,
    /// The day month year option.
    DayMonthYear,
    /// The month day year option.
    MonthDayYear,
}

/// Why a typed date/time string was rejected. [`Display`](std::fmt::Display)
/// gives a message suitable for showing next to the field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateTimeParseError {
    /// The empty option.
    Empty,
    /// The text does not match the locale's layout; `expected` is an example.
    Format {
        /// The value value.
        expected: String,
    },
    /// The invalid year option.
    InvalidYear,
    /// The invalid month option.
    InvalidMonth,
    /// The day does not exist in that month; `max` is the month's last day.
    InvalidDay {
        /// The value value.
        max: Option<u8>,
    },
    /// The invalid time option.
    InvalidTime,
}

impl std::fmt::Display for DateTimeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("Enter a date and time."),
            Self::Format { expected } => write!(f, "Use the format {expected}."),
            Self::InvalidYear => write!(f, "Year must be between {MIN_YEAR} and {MAX_YEAR}."),
            Self::InvalidMonth => f.write_str("Month must be between 1 and 12."),
            Self::InvalidDay { max: Some(max) } => {
                write!(f, "That month has only {max} days.")
            }
            Self::InvalidDay { max: None } => f.write_str("That day does not exist."),
            Self::InvalidTime => f.write_str("Enter a valid time."),
        }
    }
}

impl std::error::Error for DateTimeParseError {}

/// Hook that turns a value into display text, replacing the locale format.
pub type DateTimeFormatter = Arc<dyn Fn(DateTimeValue) -> String + Send + Sync>;
/// Hook that parses typed text, replacing the locale parser.
pub type DateTimeParser =
    Arc<dyn Fn(&str) -> Result<DateTimeValue, DateTimeParseError> + Send + Sync>;

/// Date/time layout conventions for one locale: field order, separator and
/// 12/24-hour clock.
///
/// Ownership: the app owns the locale and the timezone. The picker edits a
/// timezone-less wall-clock [`DateTimeValue`]; it never reads the system locale
/// or zone, and converting to or from an instant is the app's job. Pick a
/// preset or build one, and use [`DateTimePicker::formatter`] /
/// [`DateTimePicker::parser`] for anything this struct cannot express.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTimeLocale {
    /// The tag value.
    pub tag: SharedString,
    /// The date order value.
    pub date_order: DateOrder,
    /// The date separator value.
    pub date_separator: char,
    /// The hour12 value.
    pub hour12: bool,
    /// The am label value.
    pub am_label: SharedString,
    /// The pm label value.
    pub pm_label: SharedString,
}

impl Default for DateTimeLocale {
    fn default() -> Self {
        Self::iso()
    }
}

impl DateTimeLocale {
    /// `2026-05-25 21:30`, the picker's default.
    pub fn iso() -> Self {
        Self {
            tag: "und".into(),
            date_order: DateOrder::YearMonthDay,
            date_separator: '-',
            hour12: false,
            am_label: "AM".into(),
            pm_label: "PM".into(),
        }
    }

    /// `05/25/2026 9:30 PM`
    pub fn en_us() -> Self {
        Self {
            tag: "en-US".into(),
            date_order: DateOrder::MonthDayYear,
            date_separator: '/',
            hour12: true,
            ..Self::iso()
        }
    }

    /// `25/05/2026 21:30`
    pub fn en_gb() -> Self {
        Self {
            tag: "en-GB".into(),
            date_order: DateOrder::DayMonthYear,
            date_separator: '/',
            ..Self::iso()
        }
    }

    /// `25.05.2026 21:30`
    pub fn de_de() -> Self {
        Self {
            tag: "de-DE".into(),
            date_order: DateOrder::DayMonthYear,
            date_separator: '.',
            ..Self::iso()
        }
    }

    /// The date segments in display order, followed by hour and minute.
    pub fn segment_order(&self) -> [DateTimeSegment; 5] {
        use DateTimeSegment::*;
        match self.date_order {
            DateOrder::YearMonthDay => [Year, Month, Day, Hour, Minute],
            DateOrder::DayMonthYear => [Day, Month, Year, Hour, Minute],
            DateOrder::MonthDayYear => [Month, Day, Year, Hour, Minute],
        }
    }

    /// Formats the value for display or serialization.
    pub fn format(&self, v: DateTimeValue) -> String {
        let sep = self.date_separator;
        let date = match self.date_order {
            DateOrder::YearMonthDay => {
                format!("{:04}{sep}{:02}{sep}{:02}", v.year, v.month, v.day)
            }
            DateOrder::DayMonthYear => {
                format!("{:02}{sep}{:02}{sep}{:04}", v.day, v.month, v.year)
            }
            DateOrder::MonthDayYear => {
                format!("{:02}{sep}{:02}{sep}{:04}", v.month, v.day, v.year)
            }
        };
        if self.hour12 {
            let h = match v.hour % 12 {
                0 => 12,
                h => h,
            };
            let mark = if v.hour < 12 {
                &self.am_label
            } else {
                &self.pm_label
            };
            format!("{date} {h}:{:02} {mark}", v.minute)
        } else {
            format!("{date} {:02}:{:02}", v.hour, v.minute)
        }
    }

    /// An example of the entry format, used in error messages.
    pub fn example(&self) -> String {
        self.format(DateTimeValue {
            year: 2026,
            month: 5,
            day: 25,
            hour: 21,
            minute: 30,
        })
    }

    /// Parses an input value when it has a supported format.
    pub fn parse(&self, input: &str) -> Result<DateTimeValue, DateTimeParseError> {
        let bad = || DateTimeParseError::Format {
            expected: self.example(),
        };
        let input = input.trim();
        if input.is_empty() {
            return Err(DateTimeParseError::Empty);
        }
        let (date, time) = input.split_once(char::is_whitespace).ok_or_else(bad)?;

        let nums = date
            .split(self.date_separator)
            .map(|p| p.parse::<i32>().map_err(|_| bad()))
            .collect::<Result<Vec<_>, _>>()?;
        let [a, b, c] = nums[..] else {
            return Err(bad());
        };
        let (year, month, day) = match self.date_order {
            DateOrder::YearMonthDay => (a, b, c),
            DateOrder::DayMonthYear => (c, b, a),
            DateOrder::MonthDayYear => (c, a, b),
        };

        let mut time = time.trim();
        let mut pm = None;
        if self.hour12 {
            let lower = time.to_lowercase();
            for (label, is_pm) in [(&self.am_label, false), (&self.pm_label, true)] {
                let label = label.to_lowercase();
                if lower.ends_with(&label) {
                    time = time[..time.len() - label.len()].trim_end();
                    pm = Some(is_pm);
                    break;
                }
            }
            if pm.is_none() {
                return Err(bad());
            }
        }
        let (h, m) = time.split_once(':').ok_or_else(bad)?;
        let hour: i32 = h.trim().parse().map_err(|_| bad())?;
        let minute: i32 = m.trim().parse().map_err(|_| bad())?;

        let hour = match pm {
            Some(is_pm) => {
                if !(1..=12).contains(&hour) {
                    return Err(DateTimeParseError::InvalidTime);
                }
                hour % 12 + if is_pm { 12 } else { 0 }
            }
            None => hour,
        };

        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(DateTimeParseError::InvalidYear);
        }
        if !(1..=12).contains(&month) {
            return Err(DateTimeParseError::InvalidMonth);
        }
        let max = DateTimeValue::days_in_month(year, month as u8);
        if day < 1 || max.is_none_or(|max| day > i32::from(max)) {
            return Err(DateTimeParseError::InvalidDay { max });
        }
        if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
            return Err(DateTimeParseError::InvalidTime);
        }
        DateTimeValue::new(year, month as u8, day as u8, hour as u8, minute as u8)
            .ok_or(DateTimeParseError::InvalidTime)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Available date time segment options for configuring the control.
pub enum DateTimeSegment {
    /// The year option.
    Year,
    /// The month option.
    Month,
    #[default]
    /// The day option.
    Day,
    /// The hour option.
    Hour,
    /// The minute option.
    Minute,
}

impl DateTimeSegment {
    fn index(self) -> usize {
        match self {
            Self::Year => 0,
            Self::Month => 1,
            Self::Day => 2,
            Self::Hour => 3,
            Self::Minute => 4,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Year => "Year",
            Self::Month => "Month",
            Self::Day => "Day",
            Self::Hour => "Hour",
            Self::Minute => "Minute",
        }
    }

    fn value(self, value: DateTimeValue) -> String {
        match self {
            Self::Year => format!("{:04}", value.year),
            Self::Month => format!("{:02}", value.month),
            Self::Day => format!("{:02}", value.day),
            Self::Hour => format!("{:02}", value.hour),
            Self::Minute => format!("{:02}", value.minute),
        }
    }
}

/// Local wall-clock date/time picker for workflows such as scheduling,
/// snoozing, reminders, and rule boundaries.
#[allow(clippy::type_complexity)]
pub struct DateTimePicker {
    value: Option<DateTimeValue>,
    draft: DateTimeValue,
    placeholder: SharedString,
    disabled: bool,
    error: bool,
    open: bool,
    closing: bool,
    motion_epoch: u64,
    selected_segment: DateTimeSegment,
    focus_handle: Option<FocusHandle>,
    drop_origin: Point<Pixels>,
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    observing_theme: bool,
    locale: DateTimeLocale,
    formatter: Option<DateTimeFormatter>,
    parser: Option<DateTimeParser>,
    invalid_message: Option<SharedString>,
    on_change: Option<Box<dyn Fn(Option<DateTimeValue>, &mut App) + 'static>>,
}

impl Default for DateTimePicker {
    fn default() -> Self {
        Self {
            value: None,
            draft: DateTimeValue {
                year: 2000,
                month: 1,
                day: 1,
                hour: 9,
                minute: 0,
            },
            placeholder: "Select date and time".into(),
            disabled: false,
            error: false,
            open: false,
            closing: false,
            motion_epoch: 0,
            selected_segment: DateTimeSegment::Day,
            focus_handle: None,
            drop_origin: point(px(0.0), px(0.0)),
            trigger_bounds: Arc::new(Mutex::new(None)),
            observing_theme: false,
            locale: DateTimeLocale::default(),
            formatter: None,
            parser: None,
            invalid_message: None,
            on_change: None,
        }
    }
}

impl DateTimePicker {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current value.
    pub fn value(mut self, value: DateTimeValue) -> Self {
        self.value = Some(value);
        self.draft = value;
        self
    }

    /// Sets the maybe value configuration for this control.
    pub fn maybe_value(mut self, value: Option<DateTimeValue>) -> Self {
        self.value = value;
        if let Some(value) = value {
            self.draft = value;
        }
        self
    }

    /// Sets the default value configuration for this control.
    pub fn default_value(mut self, value: DateTimeValue) -> Self {
        self.draft = value;
        self
    }

    /// Sets the placeholder configuration for this control.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the error configuration for this control.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Registers a handler for change events.
    pub fn on_change(mut self, f: impl Fn(Option<DateTimeValue>, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Set the locale used to format, parse and order segments. See
    /// [`DateTimeLocale`] for who owns locale and timezone.
    pub fn locale(mut self, locale: DateTimeLocale) -> Self {
        self.locale = locale;
        self
    }

    /// Replace the locale's display format.
    pub fn formatter(
        mut self,
        f: impl Fn(DateTimeValue) -> String + Send + Sync + 'static,
    ) -> Self {
        self.formatter = Some(Arc::new(f));
        self
    }

    /// Replace the locale's parser used by [`DateTimePicker::parse_input`].
    pub fn parser(
        mut self,
        f: impl Fn(&str) -> Result<DateTimeValue, DateTimeParseError> + Send + Sync + 'static,
    ) -> Self {
        self.parser = Some(Arc::new(f));
        self
    }

    /// The message for the last rejected [`DateTimePicker::parse_input`], if any.
    pub fn invalid_message(&self) -> Option<&SharedString> {
        self.invalid_message.as_ref()
    }

    /// Format a value the way this picker displays it.
    pub fn format_value(&self, value: DateTimeValue) -> String {
        match &self.formatter {
            Some(f) => f(value),
            None => self.locale.format(value),
        }
    }

    /// Parse typed text with the picker's parser or locale. On success the
    /// value is set and `on_change` fires; on failure the value is kept, the
    /// picker shows the error state and [`DateTimePicker::invalid_message`]
    /// holds the message.
    pub fn parse_input(
        &mut self,
        text: &str,
        cx: &mut Context<Self>,
    ) -> Result<DateTimeValue, DateTimeParseError> {
        let parsed = match &self.parser {
            Some(f) => f(text),
            None => self.locale.parse(text),
        };
        match parsed {
            Ok(value) => {
                self.value = Some(value);
                self.draft = value;
                self.invalid_message = None;
                self.error = false;
                self.emit_change(cx);
            }
            Err(ref err) => {
                self.invalid_message = Some(err.to_string().into());
                self.error = true;
            }
        }
        cx.notify();
        parsed
    }

    /// Sets the selected value configuration for this control.
    pub fn selected_value(&self) -> Option<DateTimeValue> {
        self.value
    }

    /// Sets the value value.
    pub fn set_value(&mut self, value: Option<DateTimeValue>, cx: &mut Context<Self>) {
        self.value = value;
        if let Some(value) = value {
            self.draft = value;
        }
        cx.notify();
    }

    /// Sets the disabled value.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    /// Sets the error value.
    pub fn set_error(&mut self, error: bool, cx: &mut Context<Self>) {
        self.error = error;
        cx.notify();
    }

    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn emit_change(&self, cx: &mut Context<Self>) {
        if let Some(f) = &self.on_change {
            f(self.value, cx);
        }
    }

    fn open_popup(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.closing {
            self.motion_epoch = self.motion_epoch.wrapping_add(1);
        }
        if let Some(value) = self.value {
            self.draft = value;
        }
        self.open = true;
        self.closing = false;
        cx.notify();
    }

    fn close_popup(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.closing {
            return;
        }

        self.closing = true;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
        let epoch = self.motion_epoch;
        let duration = cx.theme().motion().popup_exit_duration();

        cx.spawn(
            move |picker: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    cx.background_executor().timer(duration).await;
                    picker
                        .update(&mut cx, move |picker, cx| {
                            if picker.motion_epoch == epoch && picker.closing {
                                picker.open = false;
                                picker.closing = false;
                                cx.notify();
                            }
                        })
                        .ok();
                }
            },
        )
        .detach();

        cx.notify();
    }

    fn update_origin(&mut self, fallback: Point<Pixels>) {
        self.drop_origin = self
            .trigger_bounds
            .lock()
            .ok()
            .and_then(|slot| *slot)
            .map(|bounds| point(bounds.origin.x, bounds.origin.y + bounds.size.height))
            .unwrap_or(fallback);
    }

    fn clear_value(&mut self, cx: &mut Context<Self>) {
        self.value = None;
        self.invalid_message = None;
        self.emit_change(cx);
        cx.notify();
    }

    fn adjust_segment(&mut self, segment: DateTimeSegment, delta: i32, cx: &mut Context<Self>) {
        let current = self.value.unwrap_or(self.draft);
        let adjusted = current.adjust(segment, delta);
        self.value = Some(adjusted);
        self.invalid_message = None;
        self.draft = adjusted;
        self.selected_segment = segment;
        self.emit_change(cx);
        cx.notify();
    }

    /// Move the selection one segment along the locale's display order,
    /// wrapping at the ends.
    fn move_segment(&mut self, step: isize, cx: &mut Context<Self>) {
        let order = self.locale.segment_order();
        let at = order
            .iter()
            .position(|s| *s == self.selected_segment)
            .unwrap_or(0) as isize;
        self.selected_segment = order[(at + step).rem_euclid(5) as usize];
        cx.notify();
    }

    fn select_segment(&mut self, segment: DateTimeSegment, cx: &mut Context<Self>) {
        self.selected_segment = segment;
        cx.notify();
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if crate::focus::handle_tab_navigation(ev, window, cx) {
            cx.notify();
            return;
        }

        if self.disabled || self.closing {
            return;
        }

        match ev.keystroke.key.as_str() {
            "escape" => self.close_popup(cx),
            "enter" | "space" => {
                if self.open {
                    self.close_popup(cx);
                } else {
                    self.open_popup(cx);
                }
            }
            "left" => self.move_segment(-1, cx),
            "right" => self.move_segment(1, cx),
            "home" => {
                self.selected_segment = self.locale.segment_order()[0];
                cx.notify();
            }
            "end" => {
                self.selected_segment = self.locale.segment_order()[4];
                cx.notify();
            }
            "up" => self.adjust_segment(self.selected_segment, 1, cx),
            "down" => self.adjust_segment(self.selected_segment, -1, cx),
            "backspace" | "delete" => self.clear_value(cx),
            _ => {}
        }
    }

    fn on_mouse_down(&mut self, ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.closing {
            return;
        }

        let handle = self.focus_handle(cx);
        window.focus(&handle, cx);
        self.update_origin(point(
            ev.position.x,
            ev.position.y + px(cx.theme().components.dropdown_height),
        ));
        if self.open {
            self.close_popup(cx);
        } else {
            self.open_popup(cx);
        }
    }
}

impl Render for DateTimePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let focus_handle = self.focus_handle(cx);
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let radii = theme.radii;
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;
        let motion = theme.motion();
        let focused = focus_handle.is_focused(window);
        let fg = if self.disabled {
            colors.on_neutral_disabled
        } else if self.value.is_none() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };
        let border = if self.error {
            colors.status_error
        } else if focused {
            colors.accent
        } else {
            colors.stroke_neutral
        };
        let label = self
            .value
            .map(|v| SharedString::from(self.format_value(v)))
            .unwrap_or_else(|| self.placeholder.clone());

        let trigger_bounds_arc = Arc::clone(&self.trigger_bounds);
        let measure_trigger = canvas(
            move |bounds, _, _| {
                if let Ok(mut slot) = trigger_bounds_arc.lock() {
                    *slot = Some(bounds);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();

        let mut root = div()
            .key_context("DateTimePicker")
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(
                |picker: &mut DateTimePicker, ev: &KeyDownEvent, window, cx| {
                    picker.handle_key(ev, window, cx);
                },
            ))
            .on_action(crate::focus::focus_next)
            .on_action(crate::focus::focus_previous)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(spacing.sm))
            .w_full()
            .h(px(components.dropdown_height))
            .px(px(spacing.md))
            .rounded(px(radii.md))
            .bg(if self.disabled {
                colors.neutral_disabled
            } else {
                colors.neutral
            })
            .border_1()
            .border_color(border)
            .cursor(if self.disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            })
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_ramp(&typography.body)
                    .text_color(fg)
                    .child(label),
            )
            .child(
                svg()
                    .path("icons/chevron_down.svg")
                    .size(px(components.popup_icon_slot))
                    .text_color(fg),
            )
            .child(measure_trigger);

        if self.open && !self.disabled {
            let close_listener = cx.listener(|picker: &mut DateTimePicker, _, _, cx| {
                picker.close_popup(cx);
            });

            let popup_min_width = self
                .trigger_bounds
                .lock()
                .ok()
                .and_then(|slot| *slot)
                .map(|bounds| f32::from(bounds.size.width).max(344.0))
                .unwrap_or(344.0);

            let mut panel = div()
                .id("date-time-picker-panel")
                .flex()
                .flex_col()
                .gap(px(spacing.md))
                .min_w(px(popup_min_width))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.stroke_neutral)
                .rounded(px(radii.md))
                .shadow_md()
                .p(px(spacing.md))
                .on_mouse_down_out(close_listener);

            let mut controls = div().flex().flex_row().gap(px(spacing.sm));
            for segment in self.locale.segment_order() {
                controls = controls.child(segment_control(
                    segment,
                    self.draft,
                    segment == self.selected_segment,
                    self.closing,
                    cx,
                ));
            }

            panel = panel.child(controls);
            if let Some(message) = self.invalid_message.clone() {
                panel = panel.child(
                    div()
                        .text_ramp(&typography.caption)
                        .text_color(colors.status_error)
                        .child(message),
                );
            }
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .gap(px(spacing.sm))
                    .child(action_button(
                        "date-time-picker-clear",
                        "Clear",
                        self.closing,
                        cx.listener(|picker: &mut DateTimePicker, _: &ClickEvent, _, cx| {
                            if picker.closing {
                                return;
                            }
                            picker.clear_value(cx);
                        }),
                    ))
                    .child(action_button(
                        "date-time-picker-done",
                        "Done",
                        self.closing,
                        cx.listener(|picker: &mut DateTimePicker, _: &ClickEvent, _, cx| {
                            picker.close_popup(cx);
                        }),
                    )),
            );

            let exiting = self.closing;
            let motion_id = ("date-time-picker-panel-motion", self.motion_epoch);
            let panel = fluent_core::popup_motion_surface(motion_id, exiting, motion, panel);

            root = root.child(deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .position(self.drop_origin)
                    .snap_to_window_with_margin(px(8.))
                    .child(panel),
            ));
        }

        root
    }
}

impl Focusable for DateTimePicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle().tab_stop(true))
    }
}

fn segment_control(
    segment: DateTimeSegment,
    value: DateTimeValue,
    selected: bool,
    disabled: bool,
    cx: &mut Context<DateTimePicker>,
) -> impl IntoElement {
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let radii = theme.radii;
    let spacing = theme.spacing;
    let typography = theme.typography;
    let border = if selected {
        colors.accent
    } else {
        colors.stroke_neutral
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(spacing.xs))
        .w(px(58.0))
        .child(
            div()
                .text_ramp(&typography.caption)
                .line_height(px(typography.caption.line_height))
                .text_color(colors.on_subtle)
                .child(segment.label()),
        )
        .child(segment_step_button(segment, 1, disabled, cx))
        .child(
            div()
                .id(("date-time-picker-segment", segment.index()))
                .flex()
                .items_center()
                .justify_center()
                .w_full()
                .h(px(30.0))
                .rounded(px(radii.sm))
                .bg(if selected {
                    colors.neutral_selected
                } else {
                    colors.neutral
                })
                .border_1()
                .border_color(border)
                .text_ramp(&typography.body)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors.on_neutral)
                .cursor(if disabled {
                    CursorStyle::Arrow
                } else {
                    CursorStyle::PointingHand
                })
                .when(!disabled, |el| {
                    let hover = colors.neutral_hover;
                    el.hover(move |s| s.bg(hover)).on_click(cx.listener(
                        move |picker: &mut DateTimePicker, _: &ClickEvent, _, cx| {
                            if !picker.closing {
                                picker.select_segment(segment, cx);
                            }
                        },
                    ))
                })
                .child(segment.value(value)),
        )
        .child(segment_step_button(segment, -1, disabled, cx))
}

fn segment_step_button(
    segment: DateTimeSegment,
    delta: i32,
    disabled: bool,
    cx: &mut Context<DateTimePicker>,
) -> impl IntoElement {
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let radii = theme.radii;
    let typography = theme.typography;
    let label = if delta > 0 { "+" } else { "-" };
    let id_prefix = if delta > 0 {
        "date-time-picker-increment"
    } else {
        "date-time-picker-decrement"
    };

    div()
        .id((id_prefix, segment.index()))
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h(px(24.0))
        .rounded(px(radii.sm))
        .bg(colors.subtle)
        .text_ramp(&typography.body)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(if disabled {
            colors.on_subtle_disabled
        } else {
            colors.on_subtle
        })
        .cursor(if disabled {
            CursorStyle::Arrow
        } else {
            CursorStyle::PointingHand
        })
        .when(!disabled, |el| {
            let hover = colors.subtle_hover;
            el.hover(move |s| s.bg(hover)).on_click(cx.listener(
                move |picker: &mut DateTimePicker, _: &ClickEvent, _, cx| {
                    if !picker.closing {
                        picker.adjust_segment(segment, delta, cx);
                    }
                },
            ))
        })
        .child(label)
}

fn action_button(
    id: &'static str,
    label: &'static str,
    disabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .px(px(10.0))
        .rounded(px(4.0))
        .text_size(px(12.0))
        .cursor(if disabled {
            CursorStyle::Arrow
        } else {
            CursorStyle::PointingHand
        })
        .when(!disabled, |el| el.on_click(on_click))
        .child(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_input_round_trips() {
        let value = DateTimeValue::parse_local("2026-05-25T09:30").unwrap();

        assert_eq!(
            value,
            DateTimeValue {
                year: 2026,
                month: 5,
                day: 25,
                hour: 9,
                minute: 30,
            }
        );
        assert_eq!(value.to_local_input_value(), "2026-05-25T09:30");
        assert_eq!(value.display_value(), "2026-05-25 09:30");
    }

    #[test]
    fn validation_rejects_invalid_days_and_times() {
        assert!(DateTimeValue::new(2024, 2, 29, 23, 59).is_some());
        assert!(DateTimeValue::new(2025, 2, 29, 12, 0).is_none());
        assert!(DateTimeValue::new(2025, 13, 1, 12, 0).is_none());
        assert!(DateTimeValue::new(2025, 1, 1, 24, 0).is_none());
        assert!(DateTimeValue::new(2025, 1, 1, 0, 60).is_none());
    }

    #[test]
    fn month_and_year_adjustments_clamp_day() {
        let value = DateTimeValue::new(2024, 3, 31, 9, 0).unwrap();

        assert_eq!(
            value.adjust(DateTimeSegment::Month, -1),
            DateTimeValue::new(2024, 2, 29, 9, 0).unwrap()
        );
        assert_eq!(
            DateTimeValue::new(2024, 2, 29, 9, 0)
                .unwrap()
                .adjust(DateTimeSegment::Year, 1),
            DateTimeValue::new(2025, 2, 28, 9, 0).unwrap()
        );
    }

    #[test]
    fn minute_and_hour_adjustments_roll_dates() {
        let value = DateTimeValue::new(2026, 12, 31, 23, 59).unwrap();

        assert_eq!(
            value.adjust(DateTimeSegment::Minute, 1),
            DateTimeValue::new(2027, 1, 1, 0, 0).unwrap()
        );
        assert_eq!(
            DateTimeValue::new(2026, 1, 1, 0, 0)
                .unwrap()
                .adjust(DateTimeSegment::Hour, -1),
            DateTimeValue::new(2025, 12, 31, 23, 0).unwrap()
        );
    }

    #[test]
    fn locale_formats_and_orders_segments() {
        let v = DateTimeValue::new(2026, 5, 25, 21, 30).unwrap();
        assert_eq!(DateTimeLocale::de_de().format(v), "25.05.2026 21:30");
        assert_eq!(DateTimeLocale::en_gb().format(v), "25/05/2026 21:30");
        assert_eq!(DateTimeLocale::en_us().format(v), "05/25/2026 9:30 PM");
        assert_eq!(
            DateTimeLocale::de_de().segment_order()[0],
            DateTimeSegment::Day
        );
        assert_eq!(
            DateTimeLocale::en_us().segment_order()[..3],
            [
                DateTimeSegment::Month,
                DateTimeSegment::Day,
                DateTimeSegment::Year
            ]
        );
    }

    #[test]
    fn locale_parse_round_trips() {
        let v = DateTimeValue::new(2026, 5, 25, 0, 5).unwrap();
        for l in [
            DateTimeLocale::iso(),
            DateTimeLocale::en_us(),
            DateTimeLocale::en_gb(),
            DateTimeLocale::de_de(),
        ] {
            assert_eq!(l.parse(&l.format(v)), Ok(v), "{}", l.tag);
        }
        assert_eq!(
            DateTimeLocale::en_us().parse("05/25/2026 12:00 pm"),
            Ok(DateTimeValue::new(2026, 5, 25, 12, 0).unwrap())
        );
    }

    #[test]
    fn locale_parse_reports_invalid_input() {
        let de = DateTimeLocale::de_de();
        assert_eq!(de.parse("  "), Err(DateTimeParseError::Empty));
        assert!(matches!(
            de.parse("2026-05-25 09:30"),
            Err(DateTimeParseError::Format { .. })
        ));
        assert_eq!(
            de.parse("31.02.2026 09:30").unwrap_err().to_string(),
            "That month has only 28 days."
        );
        assert_eq!(
            de.parse("01.13.2026 09:30"),
            Err(DateTimeParseError::InvalidMonth)
        );
        assert_eq!(
            de.parse("01.01.2026 24:00"),
            Err(DateTimeParseError::InvalidTime)
        );
        assert_eq!(
            DateTimeLocale::en_us().parse("05/25/2026 13:00 PM"),
            Err(DateTimeParseError::InvalidTime)
        );
    }
}
