use chrono::{Local, NaiveDate};

/// Today's calendar date in the local timezone (no time-of-day).
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}
