use std::io::{self, Write};
use crate::models::card::{ Card, Grade };

pub enum ReviewAction {
    Graded(Grade),
    Quit,
}

pub fn review_card(card: &Card) -> ReviewAction {
    println!("{}", card.chinese);
    print!("> ");

    io::stdout().flush().expect("flushed failed"); // so "> " shows
        
    let mut guess = String::new();
    io::stdin().read_line(&mut guess).expect("failed to read line");

    println!("You typed: {guess}");

    let guess = guess.trim();
    if guess.eq_ignore_ascii_case("q") || guess.eq_ignore_ascii_case("quit") {
        return ReviewAction::Quit;
    }

    println!("Meaning: {} - Pinyin: {}", card.meaning, card.pinyin);
    if card.meaning_matches(&guess) {
        println!("Correct");
        ReviewAction::Graded(Grade::Good)
    } else {
        println!("Incorrect");
        ReviewAction::Graded(Grade::Again)
    }
}

pub fn next_interval(grade: Grade, previous: u32) -> u32 {
    match grade {
        Grade::Again => 0,
        Grade::Good => previous.max(1) * 2, // 0 -> 2, 2 -> 4, 4 -> 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::card::Grade;

    #[test]
    fn again_is_due_immediately() {
        assert_eq!(next_interval(Grade::Again, 0), 0);
    }

    #[test]
    fn good_is_three_days() {
        assert_eq!(next_interval(Grade::Good, 0), 2);
    }

    #[test]
    fn good_is_three_days_and_extra() {
        assert_eq!(next_interval(Grade::Good, 2), 4);
    }
}

