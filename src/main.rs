use std::collections::HashMap;
mod models;
mod app;
use models::deck::Deck;
use models::progress::{CardProgress, ProgressMap};

use app::review::{review_card, next_interval};

use crate::models::card::{Card, HskLevel};
const PROGRESS_PATH: &str = "data/progress.json";

fn main() {
    let deck = Deck::from_json_path("data/hsk_words.json").expect("load deck");
    let mut progress = models::progress::load(PROGRESS_PATH).expect("load progress");
    
    let session: Vec<_> = deck
        .cards
        .iter()
        .filter(|c| c.hsk == HskLevel::Hsk1)
        .collect();    
 
    for card in session {
        let previous = progress
            .get(&card.id)
            .map(|p| p.interval_days)
            .unwrap_or(0);

        let grade = review_card(&card);
        let days = next_interval(grade, previous);

        progress.insert(
            card.id.clone(),
            CardProgress { interval_days: days },
        );
        println!("{} → {} day(s)", card.id, days);
        
    }
    models::progress::save(PROGRESS_PATH, &progress).expect("save progress");

}

