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
            .enumerate()
            .map(|i| {
                let mut lines = vec![tui::text::Spans::from(i.1.as_ref())];
                for _ in 0..i.0 {
                    lines.push(tui::text::Spans::from(tui::text::Span::styled(
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                        Style::default().add_modifier(tui::style::Modifier::ITALIC),
                    )));
                }
                tui::widgets::ListItem::new(lines)
                    .style(Style::default().fg(Color::Black).bg(Color::White))
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(tui::style::Modifier::BOLD),
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
