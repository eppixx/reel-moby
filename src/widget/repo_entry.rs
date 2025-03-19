use termion::event::Key;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct RepoEntry {
    text: String,
    old_text: String,
    changed: bool,
    default_text: bool,
}

impl RepoEntry {
    pub fn new(text: Option<&str>) -> Self {
        let default_text = "edit me or select a repository";
        Self {
            text: String::from(text.unwrap_or(default_text)),
            old_text: String::from(text.unwrap_or(default_text)),
            changed: false,
            default_text: text.is_none(),
        }
    }

    pub fn get(&self) -> String {
        self.text.clone()
    }

    pub fn set(&mut self, entry: String) {
        self.text = entry.clone();
        self.old_text = entry;
    }

    pub fn render(&self, colored: bool) -> Paragraph {
        let title = match self.changed {
            true => "Repository*",
            false => "Repository",
        };

        let border_style = if colored {
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

    pub fn handle_input(&mut self, key: termion::event::Key) {
        match key {
            // Key::Char('\n') => self.confirm(), //handled in Ui
            Key::Char(c) => {
                self.text.push(c);
                self.changed = true;
                self.default_text = false;
            }
            Key::Backspace => {
                if self.default_text {
                    self.text = String::new();
                } else {
                    self.text.pop();
                }
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
