use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::Paragraph;

use crate::models::card::Card;
use crate::models::progress::ProgressMap;

enum Phase {
    Prompt, // Show Hanzi + type answer
    Reveal, // show result, pinyin and meaning
}

struct App {
    cards: Vec<Card>,
    index: usize,
    input: String,
    #[allow(dead_code)] // used in 5c/5d
    phase: Phase,
    #[allow(dead_code)] // used in 5d reveal
    last_correct: Option<bool>,
    progress: ProgressMap,
    #[allow(dead_code)] // used when saving on grade (5e)
    progress_path: String,
}

pub fn run(cards: Vec<Card>, progress: ProgressMap) -> std::io::Result<ProgressMap> {
    if cards.is_empty() {
        return Ok(progress);
    }

    let mut app = App {
        cards,
        index: 0,
        input: String::new(),
        phase: Phase::Prompt,
        last_correct: None,
        progress,
        progress_path: String::from("data/progress.json"),
    };

    // Annotate the closure return type so `?` knows the error is `io::Error`.
    ratatui::run(|mut terminal| -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| app.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && app.handle_key(key.code) {
                    break;
                }
            }
        }
        Ok(())
    })?;

    Ok(app.progress)
}

impl App {
    fn render(&self, frame: &mut ratatui::Frame) {
        let card = &self.cards[self.index];
        let body = format!(
            "[{}/{}]\n\n{}\n\n> {}\n\nEsc/q quit · type English answer",
            self.index + 1,
            self.cards.len(),
            card.chinese,
            self.input,
        );
        frame.render_widget(Paragraph::new(body), frame.area());
    }

    /// Returns `true` when the session should end.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => return true,
            KeyCode::Char('q') if self.input.is_empty() => return true,
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Backspace => {
                self.input.pop();
            }
            _ => {}
        }
        false
    }
}
