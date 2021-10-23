use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListState};

use crate::repo;

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
    changed: bool,
    opened_file: PathBuf,
}

impl ServiceSwitcher {
    pub fn new(file: &Option<PathBuf>) -> Option<Self> {
        let mut file_list = vec![
            PathBuf::from("docker-compose.yml"),
            PathBuf::from("docker-compose.yaml"),
        ];
        match &file {
            None => (),
            Some(file) => file_list.insert(0, file.clone()),
        }

        for file in file_list {
            let list = match File::open(&file) {
                Err(_) => continue,
                Ok(file) => {
                    let buf = BufReader::new(file);
                    buf.lines()
                        .map(|l| l.expect("Could not parse line"))
                        .collect()
                }
            };

            return Some(Self {
                list,
                state: ListState::default(),
                changed: false,
                opened_file: file,
            });
        }

        //could not find docker-compose file
        None
    }

    pub fn render(&mut self, colored: bool) -> (List, &mut ListState) {
        let border_style = if colored {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = match &self.changed {
            true => format!("File: *{}*", &self.opened_file.display()),
            false => format!("File: {}", &self.opened_file.display()),
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

    /// finds the next image tag in given file
    pub fn find_next_match(&mut self) -> bool {
        let current_line: usize = self.state.selected().unwrap_or(0);

        let mut i = (current_line + 1) % self.list.len();
        loop {
            if i == current_line {
                //looped through the list
                break;
            }

            //check if line matches
            if repo::match_yaml_image(&self.list[i]).is_ok() {
                self.state.select(Some(i));
                return true;
            }

            i = (i + 1) % self.list.len(); //iterate
        }

        //nothing found
        false
    }

    /// finds the previous image tag in given file
    pub fn find_previous_match(&mut self) -> bool {
        let current_line: usize = self.state.selected().unwrap_or(0);

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
            if repo::match_yaml_image(&self.list[i]).is_ok() {
                self.state.select(Some(i));
                return true;
            }

            //iterate
            i = if i == 0 { self.list.len() - 1 } else { i - 1 }
        }

        //nothing found
        false
    }

    /// return the repository from currently selected row
    pub fn extract_repo(&self) -> Result<String, Error> {
        match self.state.selected() {
            None => Err(Error::NoneSelected),
            Some(i) => match repo::match_yaml_image(&self.list[i]) {
                Err(_) => Err(Error::Parsing(String::from("Nothing found"))),
                Ok((_, repo)) => Ok(repo.to_string()),
            },
        }
    }

    /// replace currently selected line with repo and tag
    pub fn change_current_line(&mut self, repo_with_tag: String) {
        match self.state.selected() {
            None => (),
            Some(i) => match repo::match_yaml_image(&self.list[i]) {
                Err(_) => return,
                Ok((front, _)) => self.list[i] = format!("{}{}", front, repo_with_tag),
            },
        }
        self.changed = true;
    }

    /// save the currently opened file
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        let mut file = File::create(&self.opened_file)?;
        for line in &self.list {
            file.write_all(line.as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }

        self.changed = false;
        Ok(())
    }
}
