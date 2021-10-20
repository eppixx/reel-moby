use std::fmt;

use termion::event::Key;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::tags;

#[derive(Debug)]
pub enum Error {
    NoneSelected,
    NoTags,
    NoNextPage,
    NoPrevPage,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoTags => write!(f, "There are no tags"),
            Error::NoneSelected => write!(f, "No tag selected"),
            Error::NoNextPage => write!(f, "No next page available"),
            Error::NoPrevPage => write!(f, "No previous page available"),
        }
    }
}

/// used for creating a TagList
pub enum Type {
    Status(String),
    Repo(tags::Tags),
}

pub struct TagList {
    typ: Type,
    state: ListState,
}

impl TagList {
    fn new(typ: Type) -> Self {
        Self {
            typ,
            state: ListState::default(),
        }
    }

    /// create a TagList with a status message
    pub fn with_status(status: &str) -> Self {
        Self::new(Type::Status(String::from(status)))
    }

    /// create a TagList
    pub fn with_repo(name: String) -> Self {
        match tags::Tags::new(name) {
            Err(e) => Self::with_status(&format!("{}", e)),
            Ok(tags) => Self::new(Type::Repo(tags)),
        }
    }

    /// display next page if possible
    pub fn next_page(&mut self) -> Result<(), Error> {
        match &self.typ {
            Type::Status(_) => (),
            Type::Repo(tags) => match tags.next_page() {
                Err(_) => return Err(Error::NoNextPage),
                Ok(tags) => self.typ = Type::Repo(tags),
            },
        }
        Ok(())
    }

    /// display previous page if possible
    pub fn prev_page(&mut self) -> Result<(), Error> {
        match &self.typ {
            Type::Status(_) => (),
            Type::Repo(tags) => match tags.prev_page() {
                Err(_) => return Err(Error::NoPrevPage),
                Ok(tags) => self.typ = Type::Repo(tags),
            },
        }
        Ok(())
    }

    /// get a list of tag names with info
    fn print_lines(&self) -> Vec<String> {
        match &self.typ {
            Type::Status(line) => vec![line.to_string()],
            Type::Repo(tags) => tags.results.iter().map(|r| format!("{}", r)).collect(),
        }
    }

    /// get the list of tag names
    pub fn get_names(&self) -> Result<Vec<String>, Error> {
        match &self.typ {
            Type::Status(_) => Err(Error::NoTags),
            Type::Repo(tags) => Ok(tags.results.iter().map(|r| r.tag_name.clone()).collect()),
        }
    }

    /// get the selected tag or return an error
    pub fn get_selected(&self) -> Result<String, Error> {
        match &self.typ {
            Type::Status(_) => Err(Error::NoTags),
            Type::Repo(_) => match self.state.selected() {
                None => Err(Error::NoneSelected),
                Some(i) => Ok(self.get_names().unwrap()[i].clone()),
            },
        }
    }

    pub fn render(&mut self, colored: bool) -> (List, &mut ListState) {
        let border_style = if colored {
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

    pub fn handle_input(&mut self, key: termion::event::Key) {
        match key {
            Key::Down => self.next(),
            Key::Up => self.previous(),
            _ => (),
        }
    }

    /// select next tag
    pub fn next(&mut self) {
        match self.state.selected() {
            None if self.print_lines().len() > 0 => self.state.select(Some(0)),
            None => (),
            Some(i) if i == self.print_lines().len() - 1 => self.state.select(Some(0)),
            Some(i) => self.state.select(Some(i + 1)),
        }
    }

    /// select previous tag
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
