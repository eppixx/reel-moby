use termion::event::Key;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::tags;
use crate::ui::State;

pub struct TagList {
    typ: Type,
    state: ListState,
}

#[derive(Debug)]
pub enum Error {
    NoTags,
}

pub enum Type {
    Status(String),
    Repo(tags::Tags),
}

impl TagList {
    pub fn new(typ: Type) -> Self {
        Self {
            typ,
            state: ListState::default(),
        }
    }

    pub fn with_status(status: &str) -> Self {
        Self::new(Type::Status(String::from(status)))
    }

    pub fn with_repo(name: String) -> Self {
        match tags::Tags::new(name) {
            Err(_) => Self::with_status("Couldn't query tags: no images found"),
            Ok(tags) => Self::new(Type::Repo(tags)),
        }
    }

    fn print_lines(&self) -> Vec<String> {
        match &self.typ {
            Type::Status(line) => vec![line.to_string()],
            Type::Repo(tags) => tags.results.iter().map(|r| format!("{}", r)).collect(),
        }
    }

    pub fn get_names(&self) -> Result<Vec<String>, Error> {
        match &self.typ {
            Type::Status(_) => Err(Error::NoTags),
            Type::Repo(tags) => Ok(tags.results.iter().map(|r| r.tag_name.clone()).collect()),
        }
    }

    pub fn get_selected(&self) -> Result<String, Error> {
        match &self.typ {
            Type::Status(_) => Err(Error::NoTags),
            Type::Repo(_) => match self.state.selected() {
                None => Err(Error::NoTags),
                Some(i) => Ok(self.get_names().unwrap()[i].clone()),
            },
        }
    }

    pub fn render(&mut self, state: &State) -> (List, &mut ListState) {
        let border_style = if state == &State::SelectTag {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let lines = match &self.typ {
            Type::Status(line) => vec![line.clone()],
            Type::Repo(_) => self.print_lines(),
        };

        let items: Vec<tui::widgets::ListItem> = lines
            .iter()
            .map(|l| {
                tui::widgets::ListItem::new(l.clone())
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
            None if self.print_lines().len() > 0 => self.state.select(Some(0)),
            None => (),
            Some(i) if i == self.print_lines().len() - 1 => self.state.select(Some(0)),
            Some(i) => self.state.select(Some(i + 1)),
        }
    }

    pub fn previous(&mut self) {
        match self.state.selected() {
            None if self.print_lines().len() > 0 => {
                self.state.select(Some(self.print_lines().len()))
            }
            None => (),
            Some(i) if i == 0 => self.state.select(Some(self.print_lines().len() - 1)),
            Some(i) => self.state.select(Some(i - 1)),
        }
    }
}
