use rand::seq::SliceRandom;

use crate::models::card::{Card, Grade, HskLevel};
use crate::models::progress::{self, ProgressMap};
use crate::models::review::ReviewEntry;
use crate::models::session::{
    NEW_CARD_DAILY_LIMIT, SESSION_LIMIT, assemble_session, partition_due, remaining_again,
    requeue_again,
};
use crate::utils::date_utils::today;

pub(super) enum Phase {
    SelectLevel,
    Prompt,
    Reveal,
    Browse,
    Summary,
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
    /// Session length before any Again appends.
    pub(super) initial_len: usize,
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
            initial_len: 0,
            progress_path: String::from("data/progress.json"),
        }
    }

    /// Due reviews first, then up to the remaining daily new-card allowance.
    pub(super) fn start_session(&mut self) {
        let today = today();
        let (mut reviews, mut new) =
            partition_due(&self.all_cards, self.selected_level, &self.progress, today);
        let mut rng = rand::rng();
        reviews.shuffle(&mut rng);
        new.shuffle(&mut rng);
        let allowance =
            progress::remaining_new_allowance(&self.progress, today, NEW_CARD_DAILY_LIMIT);
        self.cards = assemble_session(reviews, new, allowance, SESSION_LIMIT);
        self.cards.shuffle(&mut rng);

        if self.cards.is_empty() {
            return;
        }

        self.index = 0;
        self.initial_len = self.cards.len();
        self.input.clear();
        self.last_correct = None;
        self.history.clear();
        self.browse_pos = 0;
        self.phase = Phase::Prompt;
    }

    /// Type-in check only — no SRS write yet (that happens on 1/2/3 in Reveal).
    pub(super) fn check_answer(&mut self) {
        let card = &self.cards[self.index];
        let correct = card.meaning_matches(&self.input);

        self.history.push(ReviewEntry {
            card_index: self.index,
            correct,
            guess: self.input.clone(),
        });
        self.browse_pos = self.history.len().saturating_sub(1);
        self.last_correct = Some(correct);
        self.phase = Phase::Reveal;
    }

    pub(super) fn apply_grade(&mut self, grade: Grade) {
        let card_id = self.cards[self.index].id.clone();
        progress::apply_review(&mut self.progress, &card_id, grade, today());
        if grade == Grade::Again {
            requeue_again(&mut self.cards, self.index);
        }
        let _ = crate::models::progress::save(&self.progress_path, &self.progress);

        let _ = self.advance_after_reveal();
    }

    /// Advance quiz index after Reveal. `true` only if the whole app should quit
    /// (not used for end-of-session; that goes to Summary).
    pub(super) fn advance_after_reveal(&mut self) -> bool {
        self.index += 1;
        if self.index >= self.cards.len() {
            self.phase = Phase::Summary;
            return false;
        }
        self.input.clear();
        self.last_correct = None;
        self.phase = Phase::Prompt;
        false
    }

    pub(super) fn resume_quiz_from_browse(&mut self) {
        self.input.clear();
        // Still waiting for 1/2/3 on the current card.
        if self.last_correct.is_some() {
            self.phase = Phase::Reveal;
        } else {
            self.phase = Phase::Prompt;
        }
    }

    pub(super) fn return_to_select(&mut self) {
        self.cards.clear();
        self.history.clear();
        self.index = 0;
        self.initial_len = 0;
        self.browse_pos = 0;
        self.input.clear();
        self.last_correct = None;
        self.phase = Phase::SelectLevel;
    }

    pub(super) fn session_counts(&self) -> (usize, usize, usize) {
        let answered = self.history.len();
        let correct = self.history.iter().filter(|e| e.correct).count();
        let wrong = answered.saturating_sub(correct);
        (answered, correct, wrong)
    }

    pub(super) fn again_remaining(&self) -> usize {
        remaining_again(self.index, self.initial_len, self.cards.len())
    }
}
