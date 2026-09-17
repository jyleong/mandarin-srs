use std::fs;
use std::path::Path;

use super::card::{ Card, CardRecord, HskLevel };

#[derive(Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}


impl Deck {
    pub fn demo() -> Self {
        let records = vec![
            CardRecord {
                chinese: String::from("你好"),
                pinyin: String::from("nǐ hǎo"),
                meaning: String::from("hello"),
                hsk: HskLevel::Hsk1,
            },
            CardRecord {
                chinese: String::from("谢谢"),
                pinyin: String::from("xièxie"),
                meaning: String::from("thank you"),
                hsk: HskLevel::Hsk1,
            },
        ];

        let cards = records.into_iter().map(CardRecord::into_card).collect();
        Self { cards }
    }

    pub fn from_json_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;
        
        let records: Vec<CardRecord> = serde_json::from_str(&text)?;
        let cards = records.into_iter().map(CardRecord::into_card).collect();
        Ok(Self { cards })
    }

}