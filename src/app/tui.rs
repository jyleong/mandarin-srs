use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};

use crate::app::review::next_interval;
use crate::models::card::{Card, Grade, HskLevel};
use crate::models::progress::{CardProgress, ProgressMap};

enum Phase {
    SelectLevel,
    Prompt, // Show Hanzi + type answer
    Reveal, // show result, pinyin and meaning
}

struct App {
    /// Full deck from disk (never filtered away).
    all_cards: Vec<Card>,
    /// Current review session (filled after level select).
    cards: Vec<Card>,
    index: usize,
    input: String,
    selected_level: HskLevel,
    phase: Phase,
    last_correct: Option<bool>,
    progress: ProgressMap,
    progress_path: String,
}

pub fn run(all_cards: Vec<Card>, progress: ProgressMap) -> std::io::Result<ProgressMap> {
    if all_cards.is_empty() {
        return Ok(progress);
    }

    let mut app = App {
        all_cards,
        cards: Vec::new(),
        index: 0,
        input: String::new(),
        selected_level: HskLevel::Hsk1,
        phase: Phase::SelectLevel,
        last_correct: None,
        progress,
        progress_path: String::from("data/progress.json"),
    };

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
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [ui] = Layout::vertical([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .margin(1)
            .areas(area);

        // Select screen has its own layout — do not index into `cards` yet.
        if matches!(self.phase, Phase::SelectLevel) {
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .spacing(1)
            .split(ui);

            self.render_select_header(frame, chunks[0]);
            self.render_select(frame, chunks[1]);
            self.render_footer(frame, chunks[2]);
            return;
        }

        let card = &self.cards[self.index];

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .spacing(1)
        .split(ui);

        self.render_header(frame, chunks[0]);
        self.render_hanzi(frame, chunks[1], card);
        match self.phase {
            Phase::SelectLevel => unreachable!(),
            Phase::Prompt => self.render_input(frame, chunks[2]),
            Phase::Reveal => self.render_reveal(frame, chunks[2], card),
        }
        self.render_footer(frame, chunks[3]);
    }

    fn render_select_header(&self, frame: &mut ratatui::Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(
                " Mandarin SRS ",
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("· "),
            Span::styled("choose a level", Style::new().fg(Color::Gray)),
        ]);
        let header = Paragraph::new(title).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::new().fg(Color::DarkGray)),
        );
        frame.render_widget(header, area);
    }

    fn render_select(&self, frame: &mut ratatui::Frame, area: Rect) {
        let levels = [
            HskLevel::Hsk1,
            HskLevel::Hsk2,
            HskLevel::Hsk3,
            HskLevel::Hsk4,
            HskLevel::Hsk5,
            HskLevel::Hsk6,
        ];

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(""));
        for level in levels {
            let n = level.as_u8();
            let count = self
                .all_cards
                .iter()
                .filter(|c| c.hsk == level)
                .count();
            let selected = level == self.selected_level;

            let label = format!("  HSK {n}   ({count} cards)");
            let line = if selected {
                Line::from(vec![
                    Span::styled(
                        " › ",
                        Style::new()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        label,
                        Style::new()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw("   "),
                    Span::styled(label, Style::new().fg(Color::Gray)),
                ])
            };
            lines.push(line);
            lines.push(Line::from("")); // breathing room between rows
        }

        let list = Paragraph::new(Text::from(lines))
            .alignment(Alignment::Left)
            .block(
                Block::bordered()
                    .title(" HSK level ")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::new().fg(Color::Cyan))
                    .padding(Padding::horizontal(2)),
            );
        frame.render_widget(list, area);
    }

    /// Filter `all_cards` → `cards`, then enter Prompt.
    fn start_session(&mut self) {
        self.cards = self
            .all_cards
            .iter()
            .filter(|c| c.hsk == self.selected_level)
            .cloned()
            .collect();

        if self.cards.is_empty() {
            // Stay on select if this level has no words.
            return;
        }

        // 5h later: shuffle here
        self.index = 0;
        self.input.clear();
        self.last_correct = None;
        self.phase = Phase::Prompt;
    }

    fn render_header(&self, frame: &mut ratatui::Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(
                " Mandarin SRS ",
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("· "),
            Span::styled(
                format!(
                    "HSK {} · card {} / {}",
                    self.selected_level.as_u8(),
                    self.index + 1,
                    self.cards.len()
                ),
                Style::new().fg(Color::Gray),
            ),
        ]);

        let header = Paragraph::new(title).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::new().fg(Color::DarkGray)),
        );
        frame.render_widget(header, area);
    }

    fn render_hanzi(&self, frame: &mut ratatui::Frame, area: Rect, card: &Card) {
        let spaced = space_cjk(&card.chinese);

        let block = Block::bordered()
            .title(" 汉字 ")
            .title_alignment(Alignment::Center)
            .border_style(Style::new().fg(Color::Cyan))
            .padding(Padding::uniform(1));

        let hanzi = Paragraph::new(Line::from(Span::styled(
            spaced,
            Style::new()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(block);

        frame.render_widget(hanzi, area);
    }

    fn render_input(&self, frame: &mut ratatui::Frame, area: Rect) {
        let caret = "▌";
        let line = Line::from(vec![
            Span::styled(
                "  English  ",
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}{caret}", self.input),
                Style::new()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let input = Paragraph::new(line)
            .block(
                Block::bordered()
                    .title(" Your answer ")
                    .border_style(Style::new().fg(Color::Yellow)),
            )
            .alignment(Alignment::Left);
        frame.render_widget(input, area);
    }

    fn render_reveal(&self, frame: &mut ratatui::Frame, area: Rect, card: &Card) {
        let correct = self.last_correct == Some(true);
        let (verdict, color) = if correct {
            (" ✓  Correct ", Color::Green)
        } else {
            (" ✗  Incorrect ", Color::Red)
        };

        let text = Text::from(vec![
            Line::from(Span::styled(
                verdict,
                Style::new()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Pinyin  ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    &card.pinyin,
                    Style::new()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Meaning ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    &card.meaning,
                    Style::new()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ]);

        let reveal = Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" Answer ")
                    .border_style(Style::new().fg(color)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(reveal, area);
    }

    fn render_footer(&self, frame: &mut ratatui::Frame, area: Rect) {
        let help = match self.phase {
            Phase::SelectLevel => {
                "↑↓ / k j move   ·   1-6 jump   ·   Enter start   ·   Esc / q quit"
            }
            Phase::Prompt => "Enter submit   ·   Backspace delete   ·   Esc / q quit",
            Phase::Reveal => "Any key next card   ·   Esc quit",
        };
        let footer = Paragraph::new(Line::from(Span::styled(
            help,
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(footer, area);
    }

    /// Returns `true` when the session should end.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        match self.phase {
            Phase::SelectLevel => self.handle_select_level_key(code),
            Phase::Prompt => self.handle_prompt_key(code),
            Phase::Reveal => self.handle_reveal_key(code),
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
                    if let Some(level) = HskLevel::from_digit(d as u8) {
                        self.selected_level = level;
                    }
                }
                false
            }
            KeyCode::Enter => {
                self.start_session();
                false
            }
            KeyCode::Esc | KeyCode::Char('q') => true,
            _ => false,
        }
    }

    fn handle_prompt_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => true,
            KeyCode::Char('q') if self.input.is_empty() => true,
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
            _ => {
                self.index += 1;
                if self.index >= self.cards.len() {
                    return true;
                }
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
        let grade = if correct {
            Grade::Good
        } else {
            Grade::Again
        };
        let days = next_interval(grade, previous);

        self.progress.insert(
            card.id.clone(),
            CardProgress {
                interval_days: days,
            },
        );

        let _ = crate::models::progress::save(&self.progress_path, &self.progress);

        self.last_correct = Some(correct);
        self.phase = Phase::Reveal;
    }
}

/// Insert spaces between characters so Hanzi reads larger in a monospace grid.
fn space_cjk(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= 1 {
        return s.to_string();
    }
    chars
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("  ")
}
