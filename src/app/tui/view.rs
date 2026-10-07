use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};

use crate::models::card::{Card, HskLevel};
use crate::models::progress;
use crate::models::session::{
    NEW_CARD_DAILY_LIMIT, SESSION_LIMIT, assemble_session, partition_due,
};
use crate::utils::date_utils::today;

use super::app::{App, Phase};

impl App {
    pub(super) fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let [ui] = Layout::vertical([Constraint::Percentage(100)])
            .flex(Flex::Center)
            .margin(1)
            .areas(area);

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

        if matches!(self.phase, Phase::Summary) {
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .spacing(1)
            .split(ui);

            self.render_select_header(frame, chunks[0]);
            self.render_summary(frame, chunks[1]);
            self.render_footer(frame, chunks[2]);
            return;
        }

        let card = match self.phase {
            Phase::Browse => {
                let entry = &self.history[self.browse_pos];
                &self.cards[entry.card_index]
            }
            _ => &self.cards[self.index],
        };

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
            Phase::SelectLevel | Phase::Summary => unreachable!(),
            Phase::Prompt => self.render_input(frame, chunks[2]),
            Phase::Reveal => self.render_reveal(frame, chunks[2], card),
            Phase::Browse => self.render_browse(frame, chunks[2], card),
        }
        self.render_footer(frame, chunks[3]);
    }

    /// `(reviews, new, total_at_level)` after the daily new-card cap.
    fn session_preview(&self, level: HskLevel) -> (usize, usize, usize) {
        let today = today();
        let total = self.all_cards.iter().filter(|c| c.hsk == level).count();
        let (reviews, new) = partition_due(&self.all_cards, level, &self.progress, today);
        let allowance =
            progress::remaining_new_allowance(&self.progress, today, NEW_CARD_DAILY_LIMIT);
        let session = assemble_session(reviews, new, allowance, SESSION_LIMIT);
        let review_n = session
            .iter()
            .filter(|c| !progress::is_new(&self.progress, &c.id))
            .count();
        let new_n = session.len() - review_n;
        (review_n, new_n, total)
    }

    fn render_select_header(&self, frame: &mut ratatui::Frame, area: Rect) {
        let subtitle = match self.phase {
            Phase::Summary => "session complete",
            _ => {
                let (reviews, new, _) = self.session_preview(self.selected_level);
                if reviews + new == 0 {
                    "nothing in today's session — pick another level"
                } else {
                    "choose a level"
                }
            }
        };
        let title = Line::from(vec![
            Span::styled(
                " Mandarin SRS ",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw("· "),
            Span::styled(subtitle, Style::new().fg(Color::Gray)),
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
            let (reviews, new, total) = self.session_preview(level);
            let selected = level == self.selected_level;

            let label = format!("  HSK {n}   ({reviews} review · {new} new / {total})");
            let line = if selected {
                Line::from(vec![
                    Span::styled(
                        " › ",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
            lines.push(Line::from(""));
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

    fn render_header(&self, frame: &mut ratatui::Frame, area: Rect) {
        let status = match self.phase {
            Phase::Browse => format!(
                "HSK {} · browse {} / {} answered",
                self.selected_level.as_u8(),
                self.browse_pos + 1,
                self.history.len()
            ),
            _ => {
                let again = self.again_remaining();
                if again == 0 {
                    format!(
                        "HSK {} · card {} / {}",
                        self.selected_level.as_u8(),
                        self.index + 1,
                        self.cards.len()
                    )
                } else {
                    format!(
                        "HSK {} · card {} / {} · {} again",
                        self.selected_level.as_u8(),
                        self.index + 1,
                        self.cards.len(),
                        again
                    )
                }
            }
        };

        let title = Line::from(vec![
            Span::styled(
                " Mandarin SRS ",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw("· "),
            Span::styled(status, Style::new().fg(Color::Gray)),
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
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
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
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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

        let guess = self.history.last().map(|e| e.guess.as_str()).unwrap_or("");

        let text = Text::from(vec![
            Line::from(Span::styled(
                verdict,
                Style::new()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            you_typed_line(guess),
            Line::from(""),
            Line::from(vec![
                Span::styled("Pinyin  ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    &card.pinyin,
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Meaning ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    &card.meaning,
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            grade_keys_line(self.last_correct == Some(true)),
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

    fn render_browse(&self, frame: &mut ratatui::Frame, area: Rect, card: &Card) {
        let entry = &self.history[self.browse_pos];
        let (verdict, color) = if entry.correct {
            (" ✓  Correct ", Color::Green)
        } else {
            (" ✗  Incorrect ", Color::Red)
        };

        let text = Text::from(vec![
            Line::from(Span::styled(
                " Browse ",
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                verdict,
                Style::new()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            you_typed_line(&entry.guess),
            Line::from(""),
            Line::from(vec![
                Span::styled("Pinyin  ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    &card.pinyin,
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Meaning ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    &card.meaning,
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]);

        let browse = Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" History ")
                    .border_style(Style::new().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(browse, area);
    }

    fn render_summary(&self, frame: &mut ratatui::Frame, area: Rect) {
        let (answered, correct, wrong) = self.session_counts();
        let level = self.selected_level.as_u8();

        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!(" HSK {level} session complete "),
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Answered  ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    answered.to_string(),
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Correct   ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    format!("✓  {correct}"),
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Wrong     ", Style::new().fg(Color::DarkGray)),
                Span::styled(
                    format!("✗  {wrong}"),
                    Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]);

        let summary = Paragraph::new(text).alignment(Alignment::Left).block(
            Block::bordered()
                .title(" Summary ")
                .title_alignment(Alignment::Center)
                .border_style(Style::new().fg(Color::Cyan))
                .padding(Padding::horizontal(2)),
        );
        frame.render_widget(summary, area);
    }

    fn render_footer(&self, frame: &mut ratatui::Frame, area: Rect) {
        let help = match self.phase {
            Phase::SelectLevel => {
                let (reviews, new, _) = self.session_preview(self.selected_level);
                if reviews + new == 0 {
                    "Nothing in today's session — pick another level   ·   Esc / Ctrl+Q quit"
                } else {
                    "↑↓ / k j move   ·   1-6 jump   ·   Enter start   ·   Esc / Ctrl+Q quit"
                }
            }
            Phase::Prompt => "Enter submit   ·   Backspace delete   ·   Esc / Ctrl+Q quit",
            Phase::Reveal => "1 Again  ·  2 Good  ·  3 Easy  ·  ← browse  ·  Esc / Ctrl+Q quit",
            Phase::Browse => "← previous   ·   → next (past end → quiz)   ·   Esc / Ctrl+Q quit",
            Phase::Summary => "Enter / any key → level select   ·   Esc / Ctrl+Q quit",
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
}

fn you_typed_line(guess: &str) -> Line<'_> {
    let shown = if guess.trim().is_empty() {
        "(empty)"
    } else {
        guess
    };
    Line::from(vec![
        Span::styled("You typed ", Style::new().fg(Color::DarkGray)),
        Span::styled(
            shown,
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn grade_key_span(label: &str, suggested: bool) -> Span<'static> {
    if suggested {
        Span::styled(
            format!(" {label} "),
            Style::new()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!(" {label} "), Style::new().fg(Color::Gray))
    }
}

fn grade_keys_line(typed_correct: bool) -> Line<'static> {
    Line::from(vec![
        grade_key_span("1 Again", !typed_correct),
        Span::raw("  "),
        grade_key_span("2 Good", typed_correct),
        Span::raw("  "),
        grade_key_span("3 Easy", false),
    ])
}

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
