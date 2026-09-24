use chrono::Days;
use rand::seq::SliceRandom;

use crate::models::review::{next_interval, ReviewEntry};
use crate::models::card::{Card, Grade, HskLevel};
use crate::models::progress::{self, CardProgress, ProgressMap};
use crate::utils::date_utils::today;

pub(super) enum Phase {
    SelectLevel,
    Prompt,
    Reveal,
    Browse,
}

pub(super) struct App {
    pub(super) all_cards: Vec<Card>,
    pub(super) cards: Vec<Card>,
    pub(super) index: usize,
    pub(super) input: String,
    pub(super) selected_level: HskLevel,
    pub(super) phase: Phase,
    pub(super) last_correct: Option<bool>,
    pub(super) history: Vec<ReviewEntry>,
    pub(super) browse_pos: usize,
    pub(super) progress: ProgressMap,
    progress_path: String,
}

impl App {
    pub(super) fn new(all_cards: Vec<Card>, progress: ProgressMap) -> Self {
        Self {
            all_cards,
            cards: Vec::new(),
            index: 0,
            input: String::new(),
            selected_level: HskLevel::Hsk1,
            phase: Phase::SelectLevel,
            last_correct: None,
            history: Vec::new(),
            browse_pos: 0,
            progress,
            progress_path: String::from("data/progress.json"),
        }
    }

    /// Filter due cards for the selected level, shuffle, take 50, enter Prompt.
    pub(super) fn start_session(&mut self) {
        let today = today();
        self.cards = self
            .all_cards
            .iter()
            .filter(|c| c.hsk == self.selected_level)
            .filter(|c| progress::is_due(&self.progress, &c.id, today))
            .cloned()
            .collect();

        if self.cards.is_empty() {
            return;
        }

        self.cards.shuffle(&mut rand::rng());
        self.cards.truncate(50);

        self.index = 0;
        self.input.clear();
        self.last_correct = None;
        self.history.clear();
        self.browse_pos = 0;
        self.phase = Phase::Prompt;
    }

    pub(super) fn submit_answer(&mut self) {
        let card = &self.cards[self.index];
        let previous = self
            .progress
            .get(&card.id)
            .map(|p| p.interval_days)
            .unwrap_or(0);

        let correct = card.meaning_matches(&self.input);
        let grade = if correct {
            Grade::Good
        } else {
            Grade::Again
        };
        let days = next_interval(grade, previous);
        let due_date = today() + Days::new(u64::from(days));

        self.progress.insert(
            card.id.clone(),
            CardProgress {
                interval_days: days,
                due_date,
            },
        );

        let _ = crate::models::progress::save(&self.progress_path, &self.progress);

        self.history.push(ReviewEntry {
            card_index: self.index,
            correct,
        });
        self.browse_pos = self.history.len().saturating_sub(1);

        self.last_correct = Some(correct);
        self.phase = Phase::Reveal;
    }

    /// Advance quiz index after Reveal. `true` = session finished.
    pub(super) fn advance_after_reveal(&mut self) -> bool {
        self.index += 1;
        if self.index >= self.cards.len() {
            return true;
        }
        self.input.clear();
        self.last_correct = None;
        self.phase = Phase::Prompt;
        false
    }

    pub(super) fn resume_quiz_from_browse(&mut self) {
        self.input.clear();
        self.last_correct = None;
        self.phase = Phase::Prompt;
    }
}
