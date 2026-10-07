mod app;
mod models;
mod utils;

use models::card::Card;
use models::deck::Deck;

const PROGRESS_PATH: &str = "data/progress.json";

fn main() -> std::io::Result<()> {
    let deck = Deck::from_json_path("data/hsk_words.json").expect("load deck");
    let progress = models::progress::load(PROGRESS_PATH).expect("load progress");
    let session: Vec<Card> = deck.cards.into_iter().collect();

    let progress = app::tui::run(session, progress)?;
    models::progress::save(PROGRESS_PATH, &progress).expect("save progress");
    Ok(())
}
