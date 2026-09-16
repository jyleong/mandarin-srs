use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Deserialize)]
pub struct Card {
    pub chinese: String, // 汉字 / the word you show
    pub pinyin: String, // Hanyu pinyin (hidden during the prompt)
    pub meaning: String, // English gloss (what you type as the answer)
    pub hsk: HskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u8")]

pub enum HskLevel {
    Hsk1,
    Hsk2,
    Hsk3,
    Hsk4,
    Hsk5,
    Hsk6,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Again, // wrong or I forgot
    Good, // right
}

impl Card {

    pub fn meaning_matches(&self, guess: &str) -> bool {
        let guess = guess.trim();
        self.meaning
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .any(|alt| alt == guess)
    }
}

impl TryFrom<u8> for HskLevel {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Hsk1),
            2 => Ok(Self::Hsk2),
            3 => Ok(Self::Hsk3),
            4 => Ok(Self::Hsk4),
            5 => Ok(Self::Hsk5),
            6 => Ok(Self::Hsk6),
            other => Err(format!("invalid hsk level: {other}")),
        }
    }
}