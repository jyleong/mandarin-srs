#[derive(Debug)]
pub struct Card {
    pub chinese: String, // 汉字 / the word you show
    pub pinyin: String, // Hanyu pinyin (hidden during the prompt)
    pub meaning: String, // English gloss (what you type as the answer)
    pub hsk: HskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        guess.trim() == self.meaning
    }
}