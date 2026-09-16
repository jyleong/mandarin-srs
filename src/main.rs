mod models;
mod app;
use models::deck::Deck;
use models::progress::CardProgress;

use app::review::{review_card, next_interval};

fn main() {
    let deck = Deck::from_json_path("data/hsk_words.json").expect("load deck");

    let mut progress: Vec<CardProgress> = deck.cards.iter().map(|_| CardProgress {interval_days: 0}).collect();

    for (i, card) in deck.cards.iter().enumerate() {
        let grade = review_card(card);
        let days = next_interval(grade, 0);
        progress[i].interval_days = days;
        println!("progress[{i}] = {} day(s)", progress[i].interval_days);
    }

}
