use tui::style::{Color, Style};
use tui::widgets::{Block, List, ListItem};

pub struct Info {
    info: String,
    keys: String,
}

impl Info {
    pub fn new(info: &str) -> Self {
        Self {
            info: String::from(info),
            keys: String::from(
                "Tab Cycle widgets   C-s Save   C-r Reload   C-q Quit   C-n Next page   C-p Previous page   ↑ ↓ Select tags or image line",
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

    pub fn set_info(&mut self, info: &str) {
        self.info = String::from(info);
    }
}
