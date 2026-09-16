use std::collections::HashMap;

#[derive(Debug)]
pub struct CardProgress {
    pub interval_days: u32,
}

// card.id -> learning state

pub type ProgressMap = HashMap<String, CardProgress>;

impl CardProgress {
    pub fn new() -> Self {
        Self {interval_days: 0}
    }
}