use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

use regex::Regex;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::ui::State;

#[derive(Debug)]
pub enum Error {
    NoneSelected,
    Parsing(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoneSelected => write!(f, "None selected"),
            Error::Parsing(s) => write!(f, "Parsing error: {}", s),
        }
    }
}

pub struct ServiceSwitcher {
    list: Vec<String>,
    state: ListState,
    regex: Regex,
    changed: bool,
}

impl ServiceSwitcher {
    pub fn new() -> Self {
        let list = match File::open("docker-compose.yml") {
            Err(e) => vec![format!("No docker-compose.yml found: {}", e)],
            Ok(file) => {
                let buf = BufReader::new(file);
                buf.lines()
                    .map(|l| l.expect("Could not parse line"))
                    .collect()
            }
        };

        Self {
            list,
            state: ListState::default(),
            regex: Regex::new(r"( *image *): *(.*):([.*]??) *").unwrap(),
            changed: false,
        }
    }

    pub fn render(&mut self, state: &State) -> (List, &mut ListState) {
        let border_style = if state == &State::SelectService {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = match &self.changed {
            true => "File: *docker-compose.yml*",
            false => "File: docker-compose.yml",
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
                    .title(title)
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
        false
    }

    //return the repository from currently selected row
    pub fn extract_repo(&self) -> Result<String, Error> {
        match self.state.selected() {
            None => return Err(Error::NoneSelected),
            Some(i) => {
                let caps = match self.regex.captures(&self.list[i]) {
                    None => return Err(Error::Parsing(String::from("Nothing found"))),
                    Some(cap) => cap,
                };
                let result: String = caps.get(2).unwrap().as_str().to_string();
                return Ok(result);
            }
        }
    }

    pub fn change_current_line(&mut self, repo_with_tag: String) {
        match self.state.selected() {
            None => (),
            Some(i) => {
                let caps = match self.regex.captures(&self.list[i]) {
                    None => return,
                    Some(cap) => cap,
                };
                let mut line = caps.get(1).unwrap().as_str().to_string();
                line.push_str(": ");
                line.push_str(&repo_with_tag);
                self.list[i] = line;
            }
        }
        self.changed = true;
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        let name = "docker-compose.yml";
        let mut file = File::create(name)?;
        for line in &self.list {
            file.write_all(line.as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }

        self.changed = false;
        Ok(())
    }
}
