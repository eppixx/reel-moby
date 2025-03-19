use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListItem};

pub struct Info {
    info: String,
    keys: String,
}

impl Info {
    pub fn new(info: &str) -> Self {
        Self {
            info: String::from(info),
            keys: String::from(
                "Tab Cycle widgets   C-s Save   C-r Reload   C-q Quit   ↑ ↓ Select tags or image line   Return Select",
            ),
        }
    }

    pub fn render(&self) -> List {
        let items = vec![
            ListItem::new(self.info.clone()),
            ListItem::new(self.keys.clone()),
        ];
        List::new(items)
            .block(Block::default())
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .highlight_style(Style::default().bg(Color::Black))
    }

    /// set a text to display
    pub fn set_text(&mut self, info: &str) {
        self.info = String::from(info);
    }

    /// print a text to display
    pub fn set_info(&mut self, text: &dyn std::fmt::Display) {
        self.info = format!("{}", text);
    }
}
