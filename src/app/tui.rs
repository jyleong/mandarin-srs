use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::Paragraph;

use crate::models::card::{Card, Grade};
use crate::app::review::next_interval;
use crate::models::progress::{ProgressMap, CardProgress};

enum Phase {
    Prompt, // Show Hanzi + type answer
    Reveal, // show result, pinyin and meaning
}

struct App {
    cards: Vec<Card>,
    index: usize,
    input: String,
    phase: Phase,
    last_correct: Option<bool>,
    progress: ProgressMap,
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
        let body = match self.phase {
        Phase::Prompt => format!(
            "[{}/{}]\n\n{}\n\n> {}\n\nEnter submit · Esc/q quit",
            self.index + 1,
            self.cards.len(),
            card.chinese,
            self.input,
        ),
        Phase::Reveal => {
            let verdict = if self.last_correct == Some(true) {
                "Correct"
            } else {
                "Incorrect"
            };
            format!(
                "[{}/{}]\n\n{}\n\n{}\n{} — {}\n\nany key → next",
                self.index + 1,
                self.cards.len(),
                card.chinese,
                verdict,
                card.pinyin,
                card.meaning,
            )
        }
    };
        frame.render_widget(Paragraph::new(body), frame.area());
    }

    /// Returns `true` when the session should end.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        match self.phase {
            Phase::Prompt => self.handle_prompt_key(code),
            Phase::Reveal => self.handle_reveal_key(code), //5d stub
        }

    }

    fn handle_prompt_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => return true,
            KeyCode::Char('q') if self.input.is_empty() => return true,
            KeyCode::Enter => {
                self.submit_answer();
                false // stay in the TUI loop; just changed phase
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                false
            }
            KeyCode::Backspace => {
                self.input.pop();
                false
            }
            _ => false,
        }
    }

    fn handle_reveal_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => return true,
            _ => {
                self.index += 1;
                if self.index >= self.cards.len() { return true; } // session done
                self.input.clear();
                self.last_correct = None;
                self.phase = Phase::Prompt;
                false
            }
        }
    }

    fn submit_answer(&mut self) {
        let card = &self.cards[self.index];
        let previous = self
            .progress
            .get(&card.id)
            .map(|p| p.interval_days)
            .unwrap_or(0);

        let correct = card.meaning_matches(&self.input);
        let grade = if correct { Grade::Good } else { Grade::Again };
        let days = next_interval(grade, previous);

        self.progress.insert(
            card.id.clone(),
            CardProgress { interval_days: days },
        );
        // 5e: models::progress::save(&self.progress_path, &self.progress).ok();

        self.last_correct = Some(correct);
        self.phase = Phase::Reveal;
    }
}
