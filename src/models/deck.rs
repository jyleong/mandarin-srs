use std::fs;
use std::path::Path;

use super::card::Card;

#[derive(Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn from_json_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;

        let cards: Vec<Card> = serde_json::from_str(&text)?;
        Ok(Self { cards })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_deck_keeps_optional_example() {
        let deck = Deck::from_json_path("data/hsk_words.json").unwrap();
        let baba = deck
            .cards
            .iter()
            .find(|c| c.chinese == "爸爸")
            .expect("爸爸 in HSK1");
        assert_eq!(baba.example.as_deref(), Some("爸爸，我肚子饿了。"));
        assert_eq!(baba.example_en.as_deref(), Some("Dad, I'm hungry."));
    }
}
