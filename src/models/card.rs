use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Clone, Deserialize)]
pub struct Card {
    pub chinese: String, // 汉字 / the word you show
    pub pinyin: String,  // Hanyu pinyin (hidden during the prompt)
    pub meaning: String, // English gloss (what you type as the answer)
    #[serde(alias = "hsk")]
    pub level: TocflLevel,
    /// Optional usage sentence (汉字). Hidden until Reveal.
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub example_en: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u8")]

pub enum TocflLevel {
    Tocfl1,
    Tocfl2,
    Tocfl3,
    Tocfl4,
    Tocfl5,
    /// Official Novice 1–2 plus HSK-only leftovers.
    Extra,
}

impl TocflLevel {
    pub const ALL: [Self; 6] = [
        Self::Tocfl1,
        Self::Tocfl2,
        Self::Tocfl3,
        Self::Tocfl4,
        Self::Tocfl5,
        Self::Extra,
    ];

    pub fn as_u8(self) -> u8 {
        match self {
            Self::Tocfl1 => 1,
            Self::Tocfl2 => 2,
            Self::Tocfl3 => 3,
            Self::Tocfl4 => 4,
            Self::Tocfl5 => 5,
            Self::Extra => 0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Tocfl1 => "TOCFL 1",
            Self::Tocfl2 => "TOCFL 2",
            Self::Tocfl3 => "TOCFL 3",
            Self::Tocfl4 => "TOCFL 4",
            Self::Tocfl5 => "TOCFL 5",
            Self::Extra => "Extra",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Tocfl1 => Self::Tocfl2,
            Self::Tocfl2 => Self::Tocfl3,
            Self::Tocfl3 => Self::Tocfl4,
            Self::Tocfl4 => Self::Tocfl5,
            Self::Tocfl5 => Self::Extra,
            Self::Extra => Self::Tocfl1,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Tocfl1 => Self::Extra,
            Self::Tocfl2 => Self::Tocfl1,
            Self::Tocfl3 => Self::Tocfl2,
            Self::Tocfl4 => Self::Tocfl3,
            Self::Tocfl5 => Self::Tocfl4,
            Self::Extra => Self::Tocfl5,
        }
    }

    pub fn from_digit(d: u8) -> Option<Self> {
        match d {
            1 => Some(Self::Tocfl1),
            2 => Some(Self::Tocfl2),
            3 => Some(Self::Tocfl3),
            4 => Some(Self::Tocfl4),
            5 => Some(Self::Tocfl5),
            0 | 6 => Some(Self::Extra),
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
    pub fn id(&self) -> String {
        match self.level {
            TocflLevel::Extra => format!("extra:{}", self.chinese),
            _ => format!("tocfl{}:{}", self.level.as_u8(), self.chinese),
        }
    }

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
            chinese: "x".into(),
            pinyin: "x".into(),
            meaning: meaning.into(),
            level: TocflLevel::Tocfl1,
            example: None,
            example_en: None,
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

impl TryFrom<u8> for TocflLevel {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Extra),
            1 => Ok(Self::Tocfl1),
            2 => Ok(Self::Tocfl2),
            3 => Ok(Self::Tocfl3),
            4 => Ok(Self::Tocfl4),
            5 => Ok(Self::Tocfl5),
            other => Err(format!("invalid TOCFL level: {other}")),
        }
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn example_fields_optional_on_old_json() {
        let raw = r#"{"chinese":"爱","pinyin":"ài","meaning":"love","hsk":1}"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.id(), "tocfl1:爱");
        assert_eq!(card.example.as_deref(), None);
        assert_eq!(card.example_en.as_deref(), None);
    }

    #[test]
    fn level_field_accepts_level_key() {
        let raw = r#"{"chinese":"爱","pinyin":"ài","meaning":"love","level":1}"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.level, TocflLevel::Tocfl1);
        assert_eq!(card.id(), "tocfl1:爱");
    }

    #[test]
    fn extra_level_uses_extra_id_prefix() {
        let raw = r#"{"chinese":"我","pinyin":"wǒ","meaning":"I; me","level":0}"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.level, TocflLevel::Extra);
        assert_eq!(card.id(), "extra:我");
        assert_eq!(card.level.label(), "Extra");
    }

    #[test]
    fn example_fields_deserialize_on_card() {
        let raw = r#"{
            "chinese":"爸爸",
            "pinyin":"bàba",
            "meaning":"dad; father",
            "hsk":1,
            "example":"爸爸，我肚子饿了。",
            "example_en":"Dad, I'm hungry."
        }"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.example.as_deref(), Some("爸爸，我肚子饿了。"));
        assert_eq!(card.example_en.as_deref(), Some("Dad, I'm hungry."));
    }
}
