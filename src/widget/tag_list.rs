use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

pub struct TagList {
    list: Vec<String>,
    state: ListState,
}

impl TagList {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            list: items,
            state: ListState::default(),
        }
    }

    pub fn render(&mut self) -> (List, &mut ListState) {
        let items: Vec<tui::widgets::ListItem> = self
            .list
            .iter()
            .map(|l| {
                tui::widgets::ListItem::new(l.as_ref())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .block(Block::default().title("Tags").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .highlight_style(Style::default().bg(Color::Black))
            .highlight_symbol(">>");

        (items, &mut self.state)
    }

    pub fn next(&mut self) {
        match self.state.selected() {
            None if self.list.len() > 0 => self.state.select(Some(0)),
            None => (),
            Some(i) if i == self.list.len() - 1 => self.state.select(Some(0)),
            Some(i) => self.state.select(Some(i + 1)),
        }
    }

    pub fn previous(&mut self) {
        match self.state.selected() {
            None if self.list.len() > 0 => self.state.select(Some(self.list.len())),
            None => (),
            Some(i) if i == 0 => self.state.select(Some(self.list.len() - 1)),
            Some(i) => self.state.select(Some(i - 1)),
        }
    }
}
