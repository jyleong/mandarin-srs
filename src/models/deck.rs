use std::fs;
use std::path::Path;

use super::card::Card;

#[derive(Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    /// Load `tocfl1.json` … `tocfl6.json` from a directory.
    pub fn from_tocfl_dir(dir: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = dir.as_ref();
        let mut cards = Vec::new();
        for n in 1u8..=6 {
            let path = dir.join(format!("tocfl{n}.json"));
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut part: Vec<Card> = serde_json::from_str(&text)?;
            cards.append(&mut part);
        }
        Ok(Self { cards })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card::TocflLevel;

    #[test]
    fn real_deck_loads_split_files_and_examples() {
        let deck = Deck::from_tocfl_dir("data").unwrap();
        assert_eq!(deck.cards.len(), 1607);

        let baba = deck
            .cards
            .iter()
            .find(|c| c.chinese == "爸爸")
            .expect("爸爸 in TOCFL 1");
        assert_eq!(baba.level, TocflLevel::Tocfl1);
        assert_eq!(baba.example.as_deref(), Some("爸爸，我肚子餓了。"));
        assert_eq!(baba.example_en.as_deref(), Some("Dad, I'm hungry."));

        let idiom = deck
            .cards
            .iter()
            .find(|c| c.chinese == "半途而廢")
            .expect("半途而廢 in TOCFL 6");
        assert_eq!(idiom.level, TocflLevel::Tocfl6);
        assert!(idiom.example.as_ref().unwrap().contains("半途而廢"));
    }

    #[test]
    fn every_card_has_an_example_using_the_headword() {
        let deck = Deck::from_tocfl_dir("data").unwrap();
        for card in &deck.cards {
            let zh = card
                .example
                .as_deref()
                .unwrap_or_else(|| panic!("missing example: {}", card.chinese));
            assert!(
                zh.contains(&card.chinese),
                "{} missing from {}",
                card.chinese,
                zh
            );
            assert!(card.example_en.as_ref().is_some_and(|s| !s.is_empty()));
        }
    }
}
