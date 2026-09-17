use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub fn load(path: impl AsRef<Path>) ->  Result<ProgressMap, Box<dyn std::error::Error>> {

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