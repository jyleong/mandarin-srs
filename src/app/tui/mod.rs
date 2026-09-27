mod app;
mod keys;
mod view;

use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::models::card::Card;
use crate::models::progress::ProgressMap;

use app::App;

/// Run the TUI. Returns the (possibly updated) progress map.
pub fn run(all_cards: Vec<Card>, progress: ProgressMap) -> std::io::Result<ProgressMap> {
    if all_cards.is_empty() {
        return Ok(progress);
    }

    let mut app = App::new(all_cards, progress);

    ratatui::run(|terminal| -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| app.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && app.handle_key(key) {
                    break;
                }
            }
        }
        Ok(())
    })?;

    Ok(app.progress)
}
