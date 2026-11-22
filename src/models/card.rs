use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Clone, Deserialize)]
pub struct Card {
    pub chinese: String, // 汉字 / the word you show
    pub pinyin: String,  // Hanyu pinyin (hidden during the prompt)
    pub meaning: String, // English gloss (what you type as the answer)
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
    /// Official 準備級 1. JSON `7`.
    Novice1,
    /// Official 準備級 2. JSON `8`.
    Novice2,
    Tocfl1,
    Tocfl2,
    Tocfl3,
    Tocfl4,
    Tocfl5,
    /// HSK-only leftovers with no TOCFL twin. JSON `0`.
    Extra,
}

impl TocflLevel {
    pub const ALL: [Self; 8] = [
        Self::Novice1,
        Self::Novice2,
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
            Self::Novice1 => 7,
            Self::Novice2 => 8,
            Self::Extra => 0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Novice1 => "Novice 1",
            Self::Novice2 => "Novice 2",
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
            Self::Novice1 => Self::Novice2,
            Self::Novice2 => Self::Tocfl1,
            Self::Tocfl1 => Self::Tocfl2,
            Self::Tocfl2 => Self::Tocfl3,
            Self::Tocfl3 => Self::Tocfl4,
            Self::Tocfl4 => Self::Tocfl5,
            Self::Tocfl5 => Self::Extra,
            Self::Extra => Self::Novice1,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Novice1 => Self::Extra,
            Self::Novice2 => Self::Novice1,
            Self::Tocfl1 => Self::Novice2,
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
            7 => Some(Self::Novice1),
            8 => Some(Self::Novice2),
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
            TocflLevel::Novice1 => format!("novice1:{}", self.chinese),
            TocflLevel::Novice2 => format!("novice2:{}", self.chinese),
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

    pub fn pinyin_matches(&self, guess: &str) -> bool {
        let guess_n = normalize_pinyin(guess);
        if guess_n.is_empty() {
            return false;
        }
        pinyin_alternatives(&self.pinyin)
            .into_iter()
            .map(|alt| normalize_pinyin(&alt))
            .any(|alt| !alt.is_empty() && alt == guess_n)
    }

    pub fn check_matches(&self, guess: &str) -> bool {
        self.meaning_matches(guess) || self.pinyin_matches(guess)
    }
}

fn normalize_answer(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn pinyin_alternatives(field: &str) -> Vec<String> {
    let mut out = Vec::new();
    for segment in field.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let keep: String = segment.chars().filter(|&c| c != '(' && c != ')').collect();
        let drop = strip_paren_bodies(segment);
        if !keep.is_empty() {
            out.push(keep.clone());
        }
        if !drop.is_empty() && drop != keep {
            out.push(drop);
        }
    }
    out
}

fn strip_paren_bodies(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

fn map_pinyin_letter(c: char) -> Option<char> {
    match c {
        'a' | 'ā' | 'á' | 'ǎ' | 'à' => Some('a'),
        'e' | 'ē' | 'é' | 'ě' | 'è' => Some('e'),
        'i' | 'ī' | 'í' | 'ǐ' | 'ì' => Some('i'),
        'o' | 'ō' | 'ó' | 'ǒ' | 'ò' => Some('o'),
        'u' | 'ū' | 'ú' | 'ǔ' | 'ù' => Some('u'),
        'ü' | 'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' => Some('v'),
        'b'..='z' => Some(c),
        _ => None,
    }
}

fn rewrite_ue_after_ln(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        let ln = bytes[i] == b'l' || bytes[i] == b'n';
        if ln && i + 2 < bytes.len() && bytes[i + 1] == b'u' && bytes[i + 2] == b'e' {
            out.push(bytes[i] as char);
            out.push('v');
            out.push('e');
            i += 3;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn normalize_pinyin(s: &str) -> String {
    let lower: String = s.chars().flat_map(char::to_lowercase).collect();
    let replaced = lower.replace("u:", "v");
    let chars: Vec<char> = replaced.chars().collect();
    let mut letters = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == 'u' && chars.get(i + 1) == Some(&'\u{0308}') {
            letters.push('v');
            i += 2;
            continue;
        }
        if matches!(
            c,
            '\u{0304}' | '\u{0301}' | '\u{030c}' | '\u{0300}' | '\u{0308}'
        ) || c.is_whitespace()
            || matches!(c, '1'..='5' | '\'' | '-' | '(' | ')')
        {
            i += 1;
            continue;
        }
        if let Some(letter) = map_pinyin_letter(c) {
            letters.push(letter);
        }
        i += 1;
    }
    rewrite_ue_after_ln(&letters)
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

#[cfg(test)]
mod pinyin_tests {
    use super::*;

    fn card_pinyin(pinyin: &str) -> Card {
        Card {
            chinese: "x".into(),
            pinyin: pinyin.into(),
            meaning: "x".into(),
            level: TocflLevel::Tocfl1,
            example: None,
            example_en: None,
        }
    }

    fn card_both(meaning: &str, pinyin: &str) -> Card {
        Card {
            chinese: "x".into(),
            pinyin: pinyin.into(),
            meaning: meaning.into(),
            level: TocflLevel::Tocfl1,
            example: None,
            example_en: None,
        }
    }

    #[test]
    fn tone_mark_numbered_and_bare_are_equal() {
        let c = card_pinyin("nǐ");
        for guess in ["nǐ", "ni3", "ni", "NI", "n i"] {
            assert!(c.pinyin_matches(guess), "{guess}");
        }
    }

    #[test]
    fn spaces_in_field_and_guess_ignored() {
        let c = card_pinyin("wǒ men");
        for guess in ["women", "wo3 men", "wo men"] {
            assert!(c.pinyin_matches(guess), "{guess}");
        }
    }

    #[test]
    fn numbered_tones_multi_syllable() {
        let c = card_pinyin("nǐhǎo");
        assert!(c.pinyin_matches("ni3hao"));
        assert!(c.pinyin_matches("ni3hao3"));
    }

    #[test]
    fn numbered_tones_spaced_field() {
        let c = card_pinyin("wǒ men");
        assert!(c.pinyin_matches("wo3men"));
        assert!(c.pinyin_matches("wo3 men2"));
    }

    #[test]
    fn ue_v_u_colon_equivalent() {
        let c = card_pinyin("nǚ");
        for guess in ["nǚ", "nv", "nu:", "nv3", "nu:3", "nü"] {
            assert!(c.pinyin_matches(guess), "{guess}");
        }
    }

    #[test]
    fn space_and_ue() {
        let c = card_pinyin("nǚ ér");
        for guess in ["nver", "nu:er", "nv er"] {
            assert!(c.pinyin_matches(guess), "{guess}");
        }
    }

    #[test]
    fn plain_u_does_not_match_ue() {
        let c = card_pinyin("nǚ");
        assert!(!c.pinyin_matches("nu"));
        assert!(!c.pinyin_matches("nu3"));
    }

    #[test]
    fn lu_does_not_match_lv() {
        let c = card_pinyin("lǚ");
        assert!(!c.pinyin_matches("lu"));
        assert!(!c.pinyin_matches("lu3"));
    }

    #[test]
    fn lue_family() {
        let c = card_pinyin("lüè");
        for guess in ["lue", "lve", "lu:e", "lüe"] {
            assert!(c.pinyin_matches(guess), "{guess}");
        }
    }

    #[test]
    fn nue_family() {
        let c = card_pinyin("nüèdài");
        for guess in ["nuedai", "nvedai", "nue4dai"] {
            assert!(c.pinyin_matches(guess), "{guess}");
        }
    }

    #[test]
    fn ue_after_jqxy_stays_ue() {
        let c = card_pinyin("jué");
        assert!(c.pinyin_matches("jue"));
        assert!(!c.pinyin_matches("jve"));
    }

    #[test]
    fn field_mixed_case() {
        let c = card_pinyin("Xīngqítiān");
        assert!(c.pinyin_matches("xingqitian"));
        assert!(c.pinyin_matches("XINGQITIAN"));
    }

    #[test]
    fn comma_alternatives() {
        let c = card_pinyin("zhèlǐ, zhèr");
        assert!(c.pinyin_matches("zheli"));
        assert!(c.pinyin_matches("zher"));
        assert!(c.pinyin_matches("zhe3r"));
    }

    #[test]
    fn comma_does_not_accept_other() {
        let c = card_pinyin("zhèlǐ, zhèr");
        assert!(!c.pinyin_matches("zhe"));
        assert!(!c.pinyin_matches("zhelizher"));
    }

    #[test]
    fn empty_guess_never_matches_pinyin() {
        let c = card_pinyin("nǐ");
        assert!(!c.pinyin_matches(""));
        assert!(!c.pinyin_matches("   "));
        assert!(!c.pinyin_matches("123"));
    }

    #[test]
    fn apostrophe_optional() {
        let c = card_pinyin("fǎn'ér");
        assert!(c.pinyin_matches("faner"));
        assert!(c.pinyin_matches("fan'er"));
        assert!(c.pinyin_matches("fan er"));
    }

    #[test]
    fn hyphen_optional() {
        let c = card_pinyin("dì-yī");
        assert!(c.pinyin_matches("diyi"));
        assert!(c.pinyin_matches("di-yi"));
    }

    #[test]
    fn parenthetical_optional() {
        let c = card_pinyin("xiǎohái(zi)");
        assert!(c.pinyin_matches("xiaohai"));
        assert!(c.pinyin_matches("xiaohaizi"));
    }

    #[test]
    fn parenthetical_plus_comma() {
        let c = card_pinyin("shǒuzhǐ(tou), zhǐtou");
        assert!(c.pinyin_matches("shouzhi"));
        assert!(c.pinyin_matches("shouzhitou"));
        assert!(c.pinyin_matches("zhitou"));
    }

    #[test]
    fn fork_does_not_match_inner_only() {
        let c = card_pinyin("chē(zi)");
        assert!(!c.pinyin_matches("zi"));
    }

    #[test]
    fn not_substring() {
        let c = card_pinyin("nǐmen");
        assert!(!c.pinyin_matches("ni"));
    }

    #[test]
    fn extra_or_missing_letters() {
        let c = card_pinyin("nǐ");
        assert!(!c.pinyin_matches("n"));
        assert!(!c.pinyin_matches("nii"));
    }

    #[test]
    fn check_matches_ors_meaning_and_pinyin() {
        let c = card_both("you", "nǐ");
        assert!(c.check_matches("you"));
        assert!(c.check_matches("ni3"));
        assert!(!c.check_matches("me"));
    }

    #[test]
    fn nfd_tone_mark() {
        let c = card_pinyin("nǐ");
        assert!(c.pinyin_matches("ni\u{030c}"));
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
            7 => Ok(Self::Novice1),
            8 => Ok(Self::Novice2),
            other => Err(format!("invalid TOCFL level: {other}")),
        }
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn example_fields_optional_on_old_json() {
        let raw = r#"{"chinese":"爱","pinyin":"ài","meaning":"love","level":1}"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.id(), "tocfl1:爱");
        assert_eq!(card.example.as_deref(), None);
        assert_eq!(card.example_en.as_deref(), None);
    }

    #[test]
    fn hsk_field_is_not_accepted() {
        let raw = r#"{"chinese":"爱","pinyin":"ài","meaning":"love","hsk":1}"#;
        assert!(serde_json::from_str::<Card>(raw).is_err());
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
        let raw = r#"{"chinese":"北京","pinyin":"Běijīng","meaning":"Beijing","level":0}"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.level, TocflLevel::Extra);
        assert_eq!(card.id(), "extra:北京");
        assert_eq!(card.level.label(), "Extra");
    }

    #[test]
    fn novice_levels_use_novice_id_prefix() {
        let n1: Card =
            serde_json::from_str(r#"{"chinese":"我","pinyin":"wǒ","meaning":"I; me","level":7}"#)
                .unwrap();
        assert_eq!(n1.level, TocflLevel::Novice1);
        assert_eq!(n1.id(), "novice1:我");
        assert_eq!(n1.level.label(), "Novice 1");

        let n2: Card =
            serde_json::from_str(r#"{"chinese":"愛","pinyin":"ài","meaning":"love","level":8}"#)
                .unwrap();
        assert_eq!(n2.level, TocflLevel::Novice2);
        assert_eq!(n2.id(), "novice2:愛");
    }

    #[test]
    fn example_fields_deserialize_on_card() {
        let raw = r#"{
            "chinese":"爸爸",
            "pinyin":"bàba",
            "meaning":"dad; father",
            "level":1,
            "example":"爸爸，我肚子饿了。",
            "example_en":"Dad, I'm hungry."
        }"#;
        let card: Card = serde_json::from_str(raw).unwrap();
        assert_eq!(card.example.as_deref(), Some("爸爸，我肚子饿了。"));
        assert_eq!(card.example_en.as_deref(), Some("Dad, I'm hungry."));
    }
}
