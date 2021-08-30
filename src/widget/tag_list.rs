use termion::event::Key;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::tags::Error;
use crate::ui::State;

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

    pub fn new_line(line: &str) -> Self {
        Self {
            list: vec![String::from(line)],
            state: ListState::default(),
        }
    }

    pub fn new_with_result(result: Result<Vec<String>, Error>) -> Self {
        match result {
            Ok(lines) => Self::new(lines),
            Err(_) => Self::new_line("Error fetching tags. Is there a typo in the Repository?"),
        }
    }

    pub fn get(&self) -> Option<String> {
        match self.state.selected() {
            None => None,
            Some(i) => Some(self.list[i].clone()),
        }
    }

    pub fn render(&mut self, state: &State) -> (List, &mut ListState) {
        let border_style = if state == &State::SelectTag {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

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
            .block(
                Block::default()
                    .title("Tags")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .highlight_style(Style::default().bg(Color::Black))
            .highlight_symbol(">>");

        (items, &mut self.state)
    }

    pub fn handle_input(&mut self, state: &State, key: termion::event::Key) {
        if state != &State::SelectTag {
            return;
        }

        match key {
            Key::Down => self.next(),
            Key::Up => self.previous(),
            _ => (),
        }
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
