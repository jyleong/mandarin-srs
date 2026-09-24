use crate::models::card::{ Grade };


/// One graded card in this session (for Browse later).
pub struct ReviewEntry {
    /// Index into `App::cards` at the time of grading.
    pub card_index: usize,
    pub correct: bool,
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

