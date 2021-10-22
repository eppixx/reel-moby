use std::fmt;

use termion::event::Key;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::tags;

pub enum Error {
    NoneSelected,
    NextPageSelected,
    SelectedStatus,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoneSelected => write!(f, "No tag selected"),
            Error::NextPageSelected => write!(f, "tried to get the next page"),
            Error::SelectedStatus => write!(f, "Status message was selected"),
        }
    }
}

enum Line {
    Status(String),
    Image(tags::Images),
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Status(s) => write!(f, "{}", s),
            Line::Image(i) => write!(f, "{}", i),
        }
    }
}

pub struct TagList {
    lines: Vec<Line>,
    state: ListState,
    tags: Option<tags::Tags>,
}

impl TagList {
    pub fn with_status(status: &str) -> Self {
        Self {
            lines: vec![Line::Status(String::from(status))],
            state: ListState::default(),
            tags: None,
        }
    }

    pub fn with_repo_name(repo: String) -> Self {
        match tags::Tags::new(repo) {
            Ok(tags) => Self::with_tags(tags),
            Err(_) => Self::with_status("input repo was not found"),
        }
    }

    pub fn with_tags(mut tags: tags::Tags) -> Self {
        let mut lines: Vec<Line> = tags
            .results
            .iter()
            .map(|r| Line::Image(r.clone()))
            .collect();

        match tags.next_page() {
            Err(_) => (),
            Ok(new_tags) => {
                lines.push(Line::Status(String::from("load more tags")));
                tags = new_tags;
            }
        };

        Self {
            lines,
            state: ListState::default(),
            tags: Some(tags),
        }
    }

    pub fn render(&mut self, colored: bool) -> (List, &mut ListState) {
        let border_style = if colored {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<tui::widgets::ListItem> = self
            .lines
            .iter()
            .map(|l| {
                tui::widgets::ListItem::new(format!("{}", l))
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

    pub fn get_selected(&mut self) -> Result<String, Error> {
        match self.state.selected() {
            None => Err(Error::NoneSelected),
            Some(i) if i == self.lines.len() - 1 => {
                self.load_next_page();
                Err(Error::NextPageSelected)
            }
            Some(i) => match &self.lines[i] {
                Line::Status(_) => Err(Error::SelectedStatus),
                Line::Image(i) => Ok(i.tag_name.clone()),
            },
        }
    }

    fn load_next_page(&mut self) {
        match &self.tags {
            Some(tags) => match tags.next_page() {
                Err(_) => (),
                Ok(new_tags) => {
                    //load new tags object
                    self.tags = Some(new_tags);

                    //remove "load next page"
                    let next_page = self.lines.pop();

                    //add tags
                    for image in &self.tags.as_ref().unwrap().results {
                        self.lines.push(Line::Image(image.clone()));
                    }

                    //readd next page
                    match self.tags.as_ref().unwrap().next_page {
                        None => (),
                        Some(_) => self.lines.push(next_page.unwrap()),
                    }
                }
            },
            None => (),
        }
    }

    /// select next tag
    fn next(&mut self) {
        match self.state.selected() {
            None if self.lines.len() > 0 => self.state.select(Some(0)),
            None => (),
            Some(i) if i == self.lines.len() - 1 => self.state.select(Some(0)),
            Some(i) => self.state.select(Some(i + 1)),
        }
    }

    /// select previous tag
    fn previous(&mut self) {
        match self.state.selected() {
            None if self.lines.len() > 0 => self.state.select(Some(self.lines.len())),
            None => (),
            Some(i) if i == 0 => self.state.select(Some(self.lines.len() - 1)),
            Some(i) => self.state.select(Some(i - 1)),
        }
    }
}
