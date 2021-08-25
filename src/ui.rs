use core::time::Duration;
use std::sync::mpsc;
use std::{io, thread};

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;

use crate::widget::Widget;

pub struct Ui {
    state: State,
    repo: crate::widget::repo_entry::RepoEntry,
    tags: crate::widget::tag_list::TagList,
}

#[derive(PartialEq, Clone)]
pub enum State {
    EditRepo,
    SelectTag,
}

impl Ui {
    pub fn new() -> Result<Self, io::Error> {
        let mut ui = Ui {
            state: State::EditRepo,
            repo: crate::widget::repo_entry::RepoEntry::new("This is a text"),
            tags: crate::widget::tag_list::TagList::new(vec![
                String::from("first"),
                String::from("second"),
                String::from("third"),
                String::from("sdfs"),
            ]),
        };

        //setup tui
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        //setup input thread
        let receiver = ui.spawn_stdin_channel();

        //core interaction loop
        'core: loop {
            //draw
            terminal.draw(|rect| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    // .margin(1)
                    .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                    .split(rect.size());

                rect.render_widget(ui.repo.render(), chunks[0]);
                let (list, state) = ui.tags.render();
                rect.render_stateful_widget(list, chunks[1], state);
            })?;

            //handle input
            match receiver.try_recv() {
                Ok(Key::Ctrl('q')) => break 'core, //quit program without saving
                Ok(Key::Ctrl('s')) => {
                    if ui.state == State::SelectTag {
                        //TODO save currently selected tag
                    }
                }
                Ok(Key::Char('\n')) => {
                    if ui.state == State::EditRepo {
                        ui.state = State::SelectTag;
                        ui.repo.confirm();
                        //TODO query tags and show them switch
                    }
                }
                Ok(Key::Down) => {
                    if ui.state == State::SelectTag {
                        //TODO select tag
                        ui.tags.next();
                    }
                }
                Ok(Key::Up) => {
                    if ui.state == State::SelectTag {
                        //TODO select tag
                        ui.tags.previous();
                    }
                }
                Ok(Key::Backspace) => {
                    ui.state = State::EditRepo;
                    ui.repo.input(Key::Backspace);
                }
                Ok(key) => {
                    ui.state = State::EditRepo;
                    ui.repo.input(key);
                }
                _ => (),
            }

            //sleep for 64ms (15 fps)
            thread::sleep(std::time::Duration::from_millis(32));
        }

        Ok(ui)
    }

    pub fn spawn_stdin_channel(&self) -> mpsc::Receiver<termion::event::Key> {
        let (tx, rx) = mpsc::channel::<termion::event::Key>();

        thread::spawn(move || loop {
            let stdin = io::stdin();
            for c in stdin.keys() {
                tx.send(c.unwrap()).unwrap();
            }
        });
        thread::sleep(std::time::Duration::from_millis(64));
        rx
    }
}
