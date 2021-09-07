use termion::event::Key;
use tui::layout::Alignment;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};

use crate::ui::State;

pub struct RepoEntry {
    text: String,
    old_text: String,
    changed: bool,
}

impl RepoEntry {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            old_text: String::from(text),
            changed: false,
        }
    }

    pub fn get(&self) -> String {
        self.text.clone()
    }

    pub fn set(&mut self, entry: String) {
        self.text = entry.clone();
        self.old_text = entry;
    }

    pub fn render(&self, state: &crate::ui::State) -> Paragraph {
        let title = match self.changed {
            true => "Repository*",
            false => "Repository",
        };

        let border_style = if state == &crate::ui::State::EditRepo {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        Paragraph::new(self.text.clone())
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
    }

    pub fn handle_input(&mut self, state: &State, key: termion::event::Key) {
        if state != &State::EditRepo {
            return;
        }

        match key {
            // Key::Char('\n') => self.confirm(), //handled in Ui
            Key::Char(c) => {
                self.text.push(c);
                self.changed = true;
            }
            Key::Backspace => {
                self.text.pop();
                self.changed = true;
            }
            Key::Esc => {
                self.text = self.old_text.clone();
                self.changed = false;
            }
            _ => (),
        }
    }

    /// set the widget to unchanged
    pub fn confirm(&mut self) {
        self.old_text = self.text.clone();
        self.changed = false;
    }
}
