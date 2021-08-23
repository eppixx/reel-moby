use termion::event::Key;
use tui::layout::Alignment;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};

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

    pub fn render(&self) -> Paragraph {
        let title = match self.changed {
            true => "Repository*",
            false => "Repository",
        };
        Paragraph::new(self.text.clone())
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
    }

    pub fn confirm(&mut self) {
        self.old_text = self.text.clone();
        self.changed = false;
    }
}

impl super::Widget for RepoEntry {
    fn input(&mut self, key: termion::event::Key) {
        match key {
            Key::Esc => {
                self.text = self.old_text.clone(); //TODO return to other structure
                self.changed = false;
            }
            Key::Backspace => {
                self.text.pop();
                self.changed = true;
            }
            Key::Char(c) => {
                self.text.push(c);
                self.changed = true;
            }
            _ => (),
        }
    }
}
