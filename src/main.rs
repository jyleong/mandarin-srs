use std::collections::HashMap;
mod models;
mod app;
use models::deck::Deck;
use models::progress::{CardProgress, ProgressMap};

use app::review::{review_card, next_interval};

fn main() {
    let deck = Deck::from_json_path("data/hsk_words.json").expect("load deck");
    let mut progress: ProgressMap = HashMap::new();
    
    for card in &deck.cards {
    let previous = progress
        .get(&card.id)
        .map(|p| p.interval_days)
        .unwrap_or(0);

    let grade = review_card(card);
    let days = next_interval(grade, previous);

    progress.insert(
        card.id.clone(),
        CardProgress { interval_days: days },
    );

    println!("{} → {} day(s)", card.id, days);
}

}
