use std::fmt;

use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListState};

use crate::repository;

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

#[derive(Clone)]
enum Line {
    Status(String),
    Image(repository::Tag),
    NextPage(String),
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Status(s) => write!(f, "{}", s),
            Line::Image(i) => write!(f, "{}", i.get_name_with_details()),
            Line::NextPage(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone)]
pub struct TagList {
    lines: Vec<Line>,
    state: ListState,
    tags: Option<repository::Repo>,
}

impl TagList {
    /// shows a text in the list and no tags
    pub fn with_status(status: &str) -> Self {
        Self {
            lines: vec![Line::Status(String::from(status))],
            state: ListState::default(),
            tags: None,
        }
    }

    /// list the tags of the repository if the input is valid
    pub async fn with_repo_name(repo: String) -> Self {
        match repository::Repo::new(&repo).await {
            Ok(tags) => Self::with_tags(tags).await,
            Err(_) => Self::with_status("input repo was not found"),
        }
    }

    /// list the tags of the input
    async fn with_tags(mut tags: repository::Repo) -> Self {
        let mut lines: Vec<Line> = tags
            .get_tags()
            .iter()
            .map(|r| Line::Image(r.clone()))
            .collect();

        match tags.next_page().await {
            None => (),
            Some(new_tags) => {
                lines.push(Line::NextPage(String::from("load more tags")));
                tags = new_tags;
            }
        };

        Self {
            lines,
            state: ListState::default(),
            tags: Some(tags),
        }
    }

    pub fn set_cursor(&mut self, state: ListState) {
        self.state = state;
    }

    pub fn get_cursor(&self) -> &ListState {
        &self.state
    }

    pub fn render(&mut self, colored: bool) -> (List, &mut ListState) {
        let border_style = if colored {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };

        let items: Vec<ratatui::widgets::ListItem> = self
            .lines
            .iter()
            .map(|l| {
                ratatui::widgets::ListItem::new(format!("{}", l))
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

    pub fn create_detail_widget(&self) -> crate::widget::details::Details {
        use crate::widget::details::Details;

        match self.state.selected() {
            None => Details::new(),
            Some(i) => match &self.lines[i] {
                Line::Image(t) => Details::with_list(t.get_details()),
                _ => Details::new(),
            },
        }
    }

    pub fn get_selected(&mut self) -> Result<String, Error> {
        match self.state.selected() {
            None => Err(Error::NoneSelected),
            Some(i) => match &self.lines[i] {
                Line::Status(_) => Err(Error::SelectedStatus),
                Line::Image(i) => Ok(i.get_name().to_string()),
                Line::NextPage(_) => Err(Error::NextPageSelected),
            },
        }
    }

    /// load new tags from the next page
    pub async fn load_next_page(&mut self) {
        let Some(tags) = &self.tags else {
            return;
        };
        let Some(new_tags) = tags.next_page().await else {
            return;
        };

        //load new tags object
        self.tags = Some(new_tags);

        //remove "load next page"
        let next_page = self.lines.pop();

        //add tags
        if let Some(tags) = &self.tags {
            for image in tags.get_tags().iter() {
                self.lines.push(Line::Image(image.clone()));
            }
        }

        //readd next page item
        if (self.tags.as_ref().unwrap().next_page().await).is_some() {
            self.lines.push(next_page.unwrap());
        }
    }

    /// select next tag
    /// returns Some when more tags need to be fetched otherwise None
    pub fn next(&mut self) -> Option<()> {
        if let Some(Line::Status(_)) = self.lines.first() {
            return None;
        }
        match self.state.selected() {
            None if !self.lines.is_empty() => self.state.select(Some(0)),
            None => (),
            Some(i) if i == self.lines.len() - 2 => return Some(()),
            // Some(i) if i == self.lines.len() - 2 => return self.load_next_page().await,
            Some(i) => self.state.select(Some(i + 1)),
        }
        None
    }

    /// select previous tag
    pub fn previous(&mut self) {
        if let Some(Line::Status(_)) = self.lines.first() {
            return;
        }
        match self.state.selected() {
            None => self.state.select(Some(0)),
            Some(0) => self.state.select(Some(self.lines.len() - 2)),
            Some(i) => self.state.select(Some(i - 1)),
        }
    }
}
