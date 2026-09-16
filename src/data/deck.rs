use super::card::{ Card, HskLevel };

#[derive(Debug)]
pub struct Deck {
    pub cards: Vec<Card>,
}


impl Deck {
    pub fn demo() -> Self {
        Self {
            cards: vec![
                Card {
                    chinese: String::from("你好"),
                    pinyin: String::from("nǐ hǎo"),
                    meaning: String::from("hello"),
                    hsk: HskLevel::Hsk1,
                },
                Card {
                    chinese: String::from("谢谢"),
                    pinyin: String::from("xièxie"),
                    meaning: String::from("thank you"),
                    hsk: HskLevel::Hsk1,
                },
            ]
        }
    }
}