use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardProgress {
    pub interval_days: u32,
    /// Calendar due date as `YYYY-MM-DD` in JSON.
    /// Missing / old files → epoch date (always due until re-reviewed).
    #[serde(default = "default_due_date")]
    pub due_date: NaiveDate,
}

fn default_due_date() -> NaiveDate {
    // 1970-01-01 → is_due is always true for migrated rows.
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid date")
}

/// card.id → learning state
pub type ProgressMap = HashMap<String, CardProgress>;

impl CardProgress {
    pub fn new() -> Self {
        Self {
            interval_days: 0,
            due_date: default_due_date(),
        }
    }

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

pub fn load(path: impl AsRef<Path>) -> Result<ProgressMap, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(serde_json::from_str(&text)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn save(path: impl AsRef<Path>, progress: &ProgressMap) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string_pretty(progress)?;
    fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("2026-09-20"));
        let back: CardProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(back.due_date, d(2026, 9, 20));
    }
}
