use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, Phase};

impl App {
    /// Returns `true` when the session should end.
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> bool {
        if is_ctrl_q(key) {
            return true;
        }

        match self.phase {
            Phase::SelectLevel => self.handle_select_level_key(key.code),
            Phase::Prompt => self.handle_prompt_key(key.code),
            Phase::Reveal => self.handle_reveal_key(key.code),
            Phase::Browse => self.handle_browse_key(key.code),
        }
    }

    fn handle_select_level_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_level = self.selected_level.next();
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_level = self.selected_level.prev();
                false
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(d) = c.to_digit(10) {
                    if let Some(level) = crate::models::card::HskLevel::from_digit(d as u8) {
                        self.selected_level = level;
                    }
                }
                false
            }
            KeyCode::Enter => {
                self.start_session();
                false
            }
            KeyCode::Esc => true,
            _ => false,
        }
    }

    fn handle_prompt_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => true,
            KeyCode::Enter => {
                self.submit_answer();
                false
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
            KeyCode::Esc => true,
            KeyCode::Left => {
                if !self.history.is_empty() {
                    self.browse_pos = self.history.len() - 1;
                    self.phase = Phase::Browse;
                }
                false
            }
            KeyCode::Right => self.advance_after_reveal(),
            _ => self.advance_after_reveal(),
        }
    }

    fn handle_browse_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => true,
            KeyCode::Left => {
                if self.browse_pos > 0 {
                    self.browse_pos -= 1;
                }
                false
            }
            KeyCode::Right => {
                if self.browse_pos + 1 < self.history.len() {
                    self.browse_pos += 1;
                } else {
                    self.resume_quiz_from_browse();
                }
                false
            }
            _ => false,
        }
    }
}

fn is_ctrl_q(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
}
