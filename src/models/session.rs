use super::card::{Card, HskLevel};
use super::progress::{self, ProgressMap};
use chrono::NaiveDate;

pub const SESSION_LIMIT: usize = 50;
pub const NEW_CARD_DAILY_LIMIT: usize = 10;

/// Due reviews first, then unseen cards, never more than `session_limit`.
/// `new_allowance` is how many new cards may still be introduced today.
pub fn assemble_session(
    reviews: Vec<Card>,
    new: Vec<Card>,
    new_allowance: usize,
    session_limit: usize,
) -> Vec<Card> {
    let reviews_take = reviews.len().min(session_limit);
    let slots_left = session_limit - reviews_take;
    let new_take = new.len().min(new_allowance).min(slots_left);

    let mut session = Vec::with_capacity(reviews_take + new_take);
    session.extend(reviews.into_iter().take(reviews_take));
    session.extend(new.into_iter().take(new_take));
    session
}

/// Reviews: this HSK level, seen, due. New: this level, never graded.
/// Future-due cards are dropped.
pub fn partition_due(
    cards: &[Card],
    level: HskLevel,
    progress: &ProgressMap,
    today: NaiveDate,
) -> (Vec<Card>, Vec<Card>) {
    let mut reviews = Vec::new();
    let mut new = Vec::new();
    for card in cards.iter().filter(|c| c.hsk == level) {
        if progress::is_new(progress, &card.id) {
            new.push(card.clone());
        } else if progress::is_due(progress, &card.id, today) {
            reviews.push(card.clone());
        }
    }
    (reviews, new)
}

/// After Again, the same card should appear later in this session.
pub fn requeue_again(cards: &mut Vec<Card>, index: usize) {
    if let Some(card) = cards.get(index).cloned() {
        cards.push(card);
    }
}

/// How many Again-queued cards are still ahead (including the current one
/// once the quiz index has entered that tail).
pub fn remaining_again(index: usize, initial_len: usize, queue_len: usize) -> usize {
    queue_len.saturating_sub(initial_len.max(index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card::HskLevel;

    fn card(hanzi: &str) -> Card {
        Card {
            id: format!("hsk1:{hanzi}"),
            chinese: hanzi.into(),
            pinyin: "x".into(),
            meaning: "x".into(),
            hsk: HskLevel::Hsk1,
        }
    }

    fn ids(cards: &[Card]) -> Vec<&str> {
        cards.iter().map(|c| c.chinese.as_str()).collect()
    }

    #[test]
    fn new_cards_respect_allowance() {
        let reviews = vec![card("旧")];
        let new = vec![card("一"), card("二"), card("三")];
        let session = assemble_session(reviews, new, 2, 50);
        assert_eq!(ids(&session), ["旧", "一", "二"]);
    }

    #[test]
    fn due_reviews_fill_the_session_before_new_cards() {
        let reviews = vec![card("A"), card("B"), card("C")];
        let new = vec![card("新")];
        let session = assemble_session(reviews, new, 10, 3);
        assert_eq!(ids(&session), ["A", "B", "C"]);
    }

    #[test]
    fn leftover_slots_take_new_up_to_allowance() {
        let reviews = vec![card("A"), card("B")];
        let new = vec![card("一"), card("二"), card("三")];
        let session = assemble_session(reviews, new, 10, 4);
        assert_eq!(ids(&session), ["A", "B", "一", "二"]);
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn partition_splits_due_reviews_and_new() {
        use crate::models::progress::CardProgress;

        let today = d(2026, 10, 6);
        let cards = vec![
            card("新"),
            card("复"),
            card("未"),
            Card {
                id: "hsk2:他".into(),
                chinese: "他".into(),
                pinyin: "x".into(),
                meaning: "x".into(),
                hsk: HskLevel::Hsk2,
            },
        ];
        let mut progress = ProgressMap::new();
        progress.insert(
            "hsk1:复".into(),
            CardProgress {
                interval_days: 2,
                due_date: today,
                introduced_on: Some(d(2026, 10, 1)),
            },
        );
        progress.insert(
            "hsk1:未".into(),
            CardProgress {
                interval_days: 8,
                due_date: d(2026, 10, 20),
                introduced_on: Some(d(2026, 9, 1)),
            },
        );

        let (reviews, new) = partition_due(&cards, HskLevel::Hsk1, &progress, today);
        assert_eq!(ids(&reviews), ["复"]);
        assert_eq!(ids(&new), ["新"]);
    }

    #[test]
    fn again_appends_the_current_card() {
        let mut cards = vec![card("A"), card("B"), card("C")];
        requeue_again(&mut cards, 0);
        assert_eq!(ids(&cards), ["A", "B", "C", "A"]);
    }

    #[test]
    fn remaining_again_is_queued_tail_while_still_in_original_session() {
        // 40-card session, 3 Again appends, still on card 12
        assert_eq!(remaining_again(12, 40, 43), 3);
    }

    #[test]
    fn remaining_again_includes_current_once_you_reach_the_queue() {
        assert_eq!(remaining_again(40, 40, 43), 3);
        assert_eq!(remaining_again(42, 40, 43), 1);
    }

    #[test]
    fn remaining_again_is_zero_when_nothing_was_requeued() {
        assert_eq!(remaining_again(5, 40, 40), 0);
    }
}
