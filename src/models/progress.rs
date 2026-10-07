use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use super::card::Grade;
use super::review::next_interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardProgress {
    pub interval_days: u32,
    /// Calendar due date as `YYYY-MM-DD` in JSON.
    /// Missing / old files → epoch date (always due until re-reviewed).
    #[serde(default = "default_due_date")]
    pub due_date: NaiveDate,
    /// First day this card received a grade. `None` on legacy rows (not new).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced_on: Option<NaiveDate>,
}

/// card.id → learning state
pub type ProgressMap = HashMap<String, CardProgress>;

impl CardProgress {
    /// Ready to show if due today or earlier.
    pub fn is_due(&self, today: NaiveDate) -> bool {
        self.due_date <= today
    }
}

/// New cards (no progress row) are always due.
pub fn is_due(progress: &ProgressMap, card_id: &str, today: NaiveDate) -> bool {
    match progress.get(card_id) {
        None => true,
        Some(p) => p.is_due(today),
    }
}

pub fn is_new(progress: &ProgressMap, card_id: &str) -> bool {
    !progress.contains_key(card_id)
}

pub fn remaining_new_allowance(
    progress: &ProgressMap,
    today: NaiveDate,
    daily_limit: usize,
) -> usize {
    let used = progress
        .values()
        .filter(|p| p.introduced_on == Some(today))
        .count();
    daily_limit.saturating_sub(used)
}

pub fn apply_review(progress: &mut ProgressMap, card_id: &str, grade: Grade, today: NaiveDate) {
    let previous = progress.get(card_id);
    let previous_interval = previous.map(|p| p.interval_days).unwrap_or(0);
    let introduced_on = match previous {
        Some(p) => p.introduced_on,
        None => Some(today),
    };
    let days = next_interval(grade, previous_interval);
    let due_date = today + Days::new(u64::from(days));
    progress.insert(
        card_id.to_string(),
        CardProgress {
            interval_days: days,
            due_date,
            introduced_on,
        },
    );
}

pub fn load(path: impl AsRef<Path>) -> Result<ProgressMap, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(serde_json::from_str(&text)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn save(
    path: impl AsRef<Path>,
    progress: &ProgressMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string_pretty(progress)?;
    fs::write(path, text)?;
    Ok(())
}

fn default_due_date() -> NaiveDate {
    // 1970-01-01 → is_due is always true for migrated rows.
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid date")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card::Grade;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn missing_progress_is_due() {
        let map = ProgressMap::new();
        assert!(is_due(&map, "hsk1:爱", d(2026, 9, 17)));
    }

    #[test]
    fn future_due_date_is_not_due() {
        let mut map = ProgressMap::new();
        map.insert(
            "hsk1:爱".into(),
            CardProgress {
                interval_days: 8,
                due_date: d(2026, 9, 25),
                introduced_on: None,
            },
        );
        assert!(!is_due(&map, "hsk1:爱", d(2026, 9, 17)));
        assert!(is_due(&map, "hsk1:爱", d(2026, 9, 25)));
    }

    #[test]
    fn round_trips_iso_date() {
        let p = CardProgress {
            interval_days: 4,
            due_date: d(2026, 9, 20),
            introduced_on: None,
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("2026-09-20"));
        let back: CardProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(back.due_date, d(2026, 9, 20));
    }

    #[test]
    fn new_allowance_is_full_when_nothing_introduced_today() {
        let today = d(2026, 10, 6);
        let mut map = ProgressMap::new();
        map.insert(
            "hsk1:爱".into(),
            CardProgress {
                interval_days: 4,
                due_date: d(2026, 10, 10),
                introduced_on: Some(d(2026, 10, 1)),
            },
        );
        assert_eq!(remaining_new_allowance(&map, today, 10), 10);
    }

    #[test]
    fn new_allowance_subtracts_cards_introduced_today() {
        let today = d(2026, 10, 6);
        let mut map = ProgressMap::new();
        map.insert(
            "hsk1:一".into(),
            CardProgress {
                interval_days: 0,
                due_date: today,
                introduced_on: Some(today),
            },
        );
        map.insert(
            "hsk1:二".into(),
            CardProgress {
                interval_days: 2,
                due_date: d(2026, 10, 8),
                introduced_on: Some(today),
            },
        );
        map.insert(
            "hsk2:三".into(),
            CardProgress {
                interval_days: 0,
                due_date: today,
                introduced_on: Some(today),
            },
        );
        assert_eq!(remaining_new_allowance(&map, today, 10), 7);
    }

    #[test]
    fn legacy_rows_without_introduced_on_do_not_count() {
        let today = d(2026, 10, 6);
        let json = r#"{"interval_days":4,"due_date":"2026-10-10"}"#;
        let p: CardProgress = serde_json::from_str(json).unwrap();
        let mut map = ProgressMap::new();
        map.insert("hsk1:爱".into(), p);
        assert_eq!(remaining_new_allowance(&map, today, 10), 10);
    }

    #[test]
    fn first_review_stamps_introduced_on() {
        let today = d(2026, 10, 6);
        let mut map = ProgressMap::new();
        apply_review(&mut map, "hsk1:爱", Grade::Good, today);
        let p = map.get("hsk1:爱").unwrap();
        assert_eq!(p.introduced_on, Some(today));
        assert_eq!(p.interval_days, 2);
        assert_eq!(p.due_date, d(2026, 10, 8));
    }

    #[test]
    fn later_review_keeps_original_introduced_on() {
        let intro = d(2026, 10, 1);
        let today = d(2026, 10, 6);
        let mut map = ProgressMap::new();
        map.insert(
            "hsk1:爱".into(),
            CardProgress {
                interval_days: 2,
                due_date: today,
                introduced_on: Some(intro),
            },
        );
        apply_review(&mut map, "hsk1:爱", Grade::Easy, today);
        let p = map.get("hsk1:爱").unwrap();
        assert_eq!(p.introduced_on, Some(intro));
        assert_eq!(p.interval_days, 8);
    }

    #[test]
    fn later_review_on_legacy_row_leaves_introduced_on_empty() {
        let today = d(2026, 10, 6);
        let mut map = ProgressMap::new();
        map.insert(
            "hsk1:爱".into(),
            CardProgress {
                interval_days: 2,
                due_date: today,
                introduced_on: None,
            },
        );
        apply_review(&mut map, "hsk1:爱", Grade::Good, today);
        assert_eq!(map.get("hsk1:爱").unwrap().introduced_on, None);
    }
}
