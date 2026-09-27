use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Card {
    pub id: String,
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

impl HskLevel {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Hsk1 => 1,
            Self::Hsk2 => 2,
            Self::Hsk3 => 3,
            Self::Hsk4 => 4,
            Self::Hsk5 => 5,
            Self::Hsk6 => 6,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Hsk1 => Self::Hsk2,
            Self::Hsk2 => Self::Hsk3,
            Self::Hsk3 => Self::Hsk4,
            Self::Hsk4 => Self::Hsk5,
            Self::Hsk5 => Self::Hsk6,
            Self::Hsk6 => Self::Hsk1,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Hsk1 => Self::Hsk6,
            Self::Hsk2 => Self::Hsk1,
            Self::Hsk3 => Self::Hsk2,
            Self::Hsk4 => Self::Hsk3,
            Self::Hsk5 => Self::Hsk4,
            Self::Hsk6 => Self::Hsk5,
        }
    }

    pub fn from_digit(d: u8) -> Option<Self> {
        match d {
            1 => Some(Self::Hsk1),
            2 => Some(Self::Hsk2),
            3 => Some(Self::Hsk3),
            4 => Some(Self::Hsk4),
            5 => Some(Self::Hsk5),
            6 => Some(Self::Hsk6),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Again, // wrong or I forgot — due today
    Good,  // right — double interval
    Easy,  // very sure — longer interval
}

impl Card {
    pub fn meaning_matches(&self, guess: &str) -> bool {
        let guess = normalize_answer(guess);
        if guess.is_empty() {
            return false;
        }
        self.meaning
            .split(';')
            .map(normalize_answer)
            .filter(|s| !s.is_empty())
            .any(|alt| alt.eq_ignore_ascii_case(&guess))
    }
}

fn normalize_answer(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod meaning_tests {
    use super::*;

    fn card(meaning: &str) -> Card {
        Card {
            id: "hsk1:x".into(),
            chinese: "x".into(),
            pinyin: "x".into(),
            meaning: meaning.into(),
            hsk: HskLevel::Hsk1,
        }
    }

    #[test]
    fn ignores_case() {
        assert!(card("hello").meaning_matches("Hello"));
    }

    #[test]
    fn collapses_whitespace() {
        assert!(card("thank you").meaning_matches("  thank   you  "));
    }

    #[test]
    fn accepts_either_semicolon_gloss() {
        let c = card("dad; father");
        assert!(c.meaning_matches("dad"));
        assert!(c.meaning_matches("Father"));
    }

    #[test]
    fn empty_guess_does_not_match() {
        assert!(!card("hello").meaning_matches("   "));
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

#[derive(Debug, Deserialize)]
pub struct CardRecord {
    pub chinese: String,
    pub pinyin: String,
    pub meaning: String,
    pub hsk: HskLevel,
}

impl CardRecord {
    pub fn into_card(self) -> Card {
        let id = format!("hsk{}:{}", self.hsk.as_u8(), self.chinese);
        Card {
            id,
            chinese: self.chinese,
            pinyin: self.pinyin,
            meaning: self.meaning,
            hsk: self.hsk,
        }
    }
}