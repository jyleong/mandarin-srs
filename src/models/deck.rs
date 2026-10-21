use std::fs;
use std::path::Path;

use super::card::Card;

#[derive(Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    /// Load Novice, TOCFL 1–5, then Extra leftovers.
    pub fn from_tocfl_dir(dir: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let dir = dir.as_ref();
        let mut cards = Vec::new();
        let names = [
            "novice1.json",
            "novice2.json",
            "tocfl1.json",
            "tocfl2.json",
            "tocfl3.json",
            "tocfl4.json",
            "tocfl5.json",
            "extra.json",
        ];
        for name in names {
            let path = dir.join(name);
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
        assert_eq!(deck.cards.len(), 7264);

        let baba = deck
            .cards
            .iter()
            .find(|c| c.chinese == "爸爸")
            .expect("爸爸 in Novice 1");
        assert_eq!(baba.level, TocflLevel::Novice1);
        assert_eq!(baba.id(), "novice1:爸爸");
        assert_eq!(baba.example.as_deref(), Some("爸爸，我肚子餓了。"));
        assert_eq!(baba.example_en.as_deref(), Some("Dad, I'm hungry."));

        let love = deck
            .cards
            .iter()
            .find(|c| c.chinese == "愛")
            .expect("愛 in Novice 2");
        assert_eq!(love.level, TocflLevel::Novice2);
        assert_eq!(love.id(), "novice2:愛");

        let mrt = deck
            .cards
            .iter()
            .find(|c| c.chinese == "捷運")
            .expect("捷運 in TOCFL 2");
        assert_eq!(mrt.level, TocflLevel::Tocfl2);

        let idiom = deck
            .cards
            .iter()
            .find(|c| c.chinese == "半途而廢")
            .expect("半途而廢 in Extra (HSK-only)");
        assert_eq!(idiom.level, TocflLevel::Extra);
        assert_eq!(idiom.id(), "extra:半途而廢");
        assert!(idiom.example.as_ref().unwrap().contains("半途而廢"));

        let mut ids = std::collections::HashSet::new();
        for card in &deck.cards {
            assert!(ids.insert(card.id()), "duplicate id {}", card.id());
        }
    }

    #[test]
    fn examples_that_exist_use_the_headword() {
        let deck = Deck::from_tocfl_dir("data").unwrap();
        let with_example = deck
            .cards
            .iter()
            .filter(|c| c.example.as_ref().is_some_and(|s| !s.is_empty()))
            .count();
        assert!(
            with_example > 1000,
            "kept inherited examples: {with_example}"
        );
        for card in &deck.cards {
            if let Some(zh) = card.example.as_deref().filter(|s| !s.is_empty()) {
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
}
