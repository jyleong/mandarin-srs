use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};

use crate::models::card::{Card, HskLevel};

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
            Phase::SelectLevel => unreachable!(),
            Phase::Prompt => self.render_input(frame, chunks[2]),
            Phase::Reveal => self.render_reveal(frame, chunks[2], card),
            Phase::Browse => self.render_browse(frame, chunks[2], card),
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
            let count = self.all_cards.iter().filter(|c| c.hsk == level).count();
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
            _ => format!(
                "HSK {} · card {} / {}",
                self.selected_level.as_u8(),
                self.index + 1,
                self.cards.len()
            ),
        };

        let title = Line::from(vec![
            Span::styled(
                " Mandarin SRS ",
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
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

        let browse = Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" History ")
                    .border_style(Style::new().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(browse, area);
    }

    fn render_footer(&self, frame: &mut ratatui::Frame, area: Rect) {
        let help = match self.phase {
            Phase::SelectLevel => {
                "↑↓ / k j move   ·   1-6 jump   ·   Enter start   ·   Esc / Ctrl+Q quit"
            }
            Phase::Prompt => "Enter submit   ·   Backspace delete   ·   Esc / Ctrl+Q quit",
            Phase::Reveal => "← browse history   ·   → / any key next card   ·   Esc / Ctrl+Q quit",
            Phase::Browse => "← previous   ·   → next (past end → quiz)   ·   Esc / Ctrl+Q quit",
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
