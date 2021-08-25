use termion::event::Key;
use tui::layout::Alignment;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

pub struct TagList {
    list: super::StatefulList<String>,
}

impl TagList {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            list: super::StatefulList::with_items(items),
        }
    }

    pub fn render(&mut self) -> (List, &mut ListState) {
        let items: Vec<tui::widgets::ListItem> = self
            .list
            .items
            .iter()
            .map(|i| {
                let lines = vec![tui::text::Spans::from(i.as_ref())];
                tui::widgets::ListItem::new(lines)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tags"))
            .highlight_style(
                Style::default().bg(Color::Black), // .add_modifier(tui::style::Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        (items, &mut self.list.state)
    }

    pub fn next(&mut self) {
        self.list.next();
    }

    pub fn previous(&mut self) {
        self.list.previous();
    }
}
