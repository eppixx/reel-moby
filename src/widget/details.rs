use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List};

use crate::repository;

pub struct Details {
    details: Vec<repository::TagDetails>,
}

impl Details {
    pub fn new() -> Self {
        Self { details: vec![] }
    }

    pub fn with_list(details: &[crate::repository::TagDetails]) -> Self {
        let mut detail = Self {
            details: details.to_owned(),
        };

        detail.details.sort_by(|a, b| a.arch.cmp(&b.arch));
        detail.details.dedup();
        detail
    }

    pub fn get_details(&self) -> Vec<String> {
        let mut lines = vec![format!("{:^10}|{:^6}|{:^6}", "ARCH", "OS", "SIZE")];
        for d in &self.details {
            lines.push(format!(
                "{:^10}|{:^6}|{:^6}",
                d.arch.clone().unwrap_or_default(),
                d.os.clone().unwrap_or_default(),
                format!("{}MB", d.size.unwrap_or_default() / 1024 / 1024)
            ));
        }
        lines
    }

    pub fn render(&self) -> List {
        let items: Vec<tui::widgets::ListItem> = self
            .get_details()
            .iter()
            .map(|l| {
                tui::widgets::ListItem::new(l.to_string())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
            })
            .collect();

        List::new(items)
            .block(
                Block::default()
                    .title("Details")
                    .borders(Borders::ALL)
                    .border_style(Style::default()),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
    }
}
