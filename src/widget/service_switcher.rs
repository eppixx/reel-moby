// use std::fs::File;
// use std::io::BufWriter;

use regex::Regex;
use termion::event::Key;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::ui::State;

pub enum Error {
    NoneSelected,
    Parsing(String),
}

pub struct ServiceSwitcher {
    list: Vec<String>,
    state: ListState,
    regex: Regex,
}

impl ServiceSwitcher {
    pub fn new() -> Self {
        let list: Vec<String> = vec![
            String::from("dies"),
            String::from("ist"),
            String::from("  image: rocketchat/rocket.chat:latest"),
            String::from("ein"),
            String::from("test"),
            String::from("  image: sdfsfdsf:latest"),
        ];
        Self {
            list,
            state: ListState::default(),
            regex: Regex::new(r"^ *image *:.*").unwrap(),
        }
    }

    pub fn render(&mut self, state: &State) -> (List, &mut ListState) {
        let border_style = if state == &State::SelectService {
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

    pub fn find_next_match(&mut self) -> bool {
        let current_line: usize = match self.state.selected() {
            None => 0,
            Some(i) => i,
        };

        let mut i = (current_line + 1) % self.list.len();
        loop {
            if i == current_line {
                //looped through the list
                break;
            }

            //check if line matches
            if self.regex.is_match(&self.list[i]) {
                self.state.select(Some(i));
                return true;
            }

            i = (i + 1) % self.list.len(); //iterate
        }

        //nothing found
        self.state.select(None);
        false
    }

    pub fn find_previous_match(&mut self) -> bool {
        let current_line: usize = match self.state.selected() {
            None => 0,
            Some(i) => i,
        };

        let mut i: usize = if current_line == 0 {
            self.list.len() - 1
        } else {
            current_line - 1
        };

        loop {
            if i == current_line {
                //looped through the list
                break;
            }

            //check if line matches
            if self.regex.is_match(&self.list[i]) {
                self.state.select(Some(i));
                return true;
            }

            //iterate
            i = if i == 0 { self.list.len() - 1 } else { i - 1 }
        }

        //nothing found
        self.state.select(None);
        false
    }

    pub fn extract_repo(&self) -> Result<String, Error> {
        let regex = Regex::new(r"( *image *): *(.*[:.*]?) *").unwrap();
        match self.state.selected() {
            None => return Err(Error::NoneSelected),
            Some(i) => {
                let caps = regex.captures(&self.list[i]).unwrap();
                let result: String = caps.get(2).unwrap().as_str().to_string();
                return Ok(result);
            }
        }
    }
}
