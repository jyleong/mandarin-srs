use std::fs;
use std::path::Path;

use super::card::{ Card, HskLevel };

#[derive(Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}


impl Deck {

    pub fn from_json_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;
        let cards: Vec<Card> = serde_json::from_str(&text)?;
        Ok(Self {cards})
    }

}