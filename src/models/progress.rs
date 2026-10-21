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

/// Rewrite `hskN:汉字` keys to `tocflN:汉字`. Returns whether anything changed.
/// On collision, keep the row with the larger `interval_days`.
pub fn migrate_hsk_keys(progress: ProgressMap) -> (ProgressMap, bool) {
    let mut changed = false;
    let mut out = ProgressMap::new();
    for (key, row) in progress {
        let new_key = if let Some(rest) = key.strip_prefix("hsk") {
            changed = true;
            format!("tocfl{rest}")
        } else {
            key
        };
        match out.get(&new_key) {
            Some(existing) if existing.interval_days >= row.interval_days => {}
            _ => {
                out.insert(new_key, row);
            }
        }
    }
    (out, changed)
}

/// Apply `old_id → new_id` (script / lexical remap). Collision: keep higher interval.
pub fn migrate_ids(progress: ProgressMap, id_map: &HashMap<String, String>) -> (ProgressMap, bool) {
    if id_map.is_empty() {
        return (progress, false);
    }
    let mut changed = false;
    let mut out = ProgressMap::new();
    for (key, row) in progress {
        let new_key = match id_map.get(&key) {
            Some(mapped) if mapped != &key => {
                changed = true;
                mapped.clone()
            }
            _ => key,
        };
        match out.get(&new_key) {
            Some(existing) if existing.interval_days >= row.interval_days => {}
            _ => {
                out.insert(new_key, row);
            }
        }
    }
    (out, changed)
}

pub fn load_id_map(
    path: impl AsRef<Path>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(serde_json::from_str(&text)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(e.into()),
    }
}

pub fn load(path: impl AsRef<Path>) -> Result<ProgressMap, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    match fs::read_to_string(path) {
        Ok(text) => {
            let map: ProgressMap = serde_json::from_str(&text)?;
            let (map, changed_hsk) = migrate_hsk_keys(map);
            let id_map_path = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("id_map.json");
            let id_map = load_id_map(id_map_path)?;
            let (map, changed_ids) = migrate_ids(map, &id_map);
            if changed_hsk || changed_ids {
                let bak = Path::new(&format!("{}.bak", path.display())).to_path_buf();
                fs::copy(path, &bak)?;
                save(path, &map)?;
            }
            Ok(map)
        }
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

    fn row(days: u32) -> CardProgress {
        CardProgress {
            interval_days: days,
            due_date: d(2026, 10, 30),
            introduced_on: None,
        }
    }

    #[test]
    fn migrate_rewrites_hsk_prefix_and_leaves_tocfl_keys() {
        let mut map = ProgressMap::new();
        map.insert("hsk1:谢谢".into(), row(128));
        map.insert("tocfl2:阴".into(), row(4));
        let (out, changed) = migrate_hsk_keys(map);
        assert!(changed);
        assert_eq!(out.get("tocfl1:谢谢").unwrap().interval_days, 128);
        assert!(out.get("hsk1:谢谢").is_none());
        assert_eq!(out.get("tocfl2:阴").unwrap().interval_days, 4);
    }

    #[test]
    fn migrate_is_noop_when_keys_already_tocfl() {
        let mut map = ProgressMap::new();
        map.insert("tocfl1:爱".into(), row(2));
        let (out, changed) = migrate_hsk_keys(map);
        assert!(!changed);
        assert_eq!(out.len(), 1);
        assert!(out.contains_key("tocfl1:爱"));
    }

    #[test]
    fn migrate_keeps_higher_interval_on_collision() {
        let mut map = ProgressMap::new();
        map.insert("hsk1:爱".into(), row(8));
        map.insert("tocfl1:爱".into(), row(32));
        let (out, _) = migrate_hsk_keys(map);
        assert_eq!(out.get("tocfl1:爱").unwrap().interval_days, 32);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn migrate_ids_rewrites_keys_from_map() {
        let mut map = ProgressMap::new();
        map.insert("tocfl1:谢谢".into(), row(128));
        map.insert("tocfl2:阴".into(), row(4));
        let mut ids = HashMap::new();
        ids.insert("tocfl1:谢谢".into(), "tocfl1:謝謝".into());
        ids.insert("tocfl2:阴".into(), "tocfl2:陰".into());
        let (out, changed) = migrate_ids(map, &ids);
        assert!(changed);
        assert_eq!(out.get("tocfl1:謝謝").unwrap().interval_days, 128);
        assert!(out.get("tocfl1:谢谢").is_none());
        assert!(out.contains_key("tocfl2:陰"));
    }

    #[test]
    fn migrate_ids_keeps_higher_interval_on_collision() {
        let mut map = ProgressMap::new();
        map.insert("tocfl1:爱".into(), row(8));
        map.insert("tocfl1:愛".into(), row(32));
        let mut ids = HashMap::new();
        ids.insert("tocfl1:爱".into(), "tocfl1:愛".into());
        let (out, _) = migrate_ids(map, &ids);
        assert_eq!(out.get("tocfl1:愛").unwrap().interval_days, 32);
        assert_eq!(out.len(), 1);
    }
}
