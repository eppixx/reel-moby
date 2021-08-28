use std::sync::mpsc;
use std::{io, thread};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

use crate::tags;
use crate::widget::repo_entry;
use crate::widget::tag_list;

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
    pub fn run(repo_id: &str) {
        let mut ui = Ui {
            state: State::EditRepo,
            repo: repo_entry::RepoEntry::new(repo_id),
            tags: tag_list::TagList::new(vec![String::from("Fetching Tags")]),
        };
        ui.tags = tag_list::TagList::new_with_result(tags::Tags::get_tags(ui.repo.get()));

        //setup tui
        let stdout = io::stdout().into_raw_mode().unwrap();
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        //setup input thread
        let receiver = ui.spawn_stdin_channel();

        //core interaction loop
        'core: loop {
            //draw
            terminal
                .draw(|rect| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                        .split(rect.size());

                    rect.render_widget(ui.repo.render(&ui.state), chunks[0]);
                    let (list, state) = ui.tags.render(&ui.state);
                    rect.render_stateful_widget(list, chunks[1], state);
                })
                .unwrap();

            //handle input
            match receiver.try_recv() {
                Ok(Key::Char('\t')) => {
                    ui.state = match ui.state {
                        State::EditRepo => State::SelectTag,
                        State::SelectTag => State::EditRepo,
                    };
                }
                Ok(Key::Ctrl('q')) => break 'core, //quit program without saving
                Ok(Key::Ctrl('s')) => {
                    if ui.state == State::SelectTag {
                        //TODO save currently selected tag
                    }
                }
                Ok(Key::Char('\n')) => {
                    if ui.state == State::EditRepo {
                        ui.repo.confirm();
                        ui.tags =
                            tag_list::TagList::new_with_result(tags::Tags::get_tags(ui.repo.get()));
                    }
                }
                Ok(Key::Char(key)) => {
                    if ui.state == State::EditRepo {
                        ui.tags = tag_list::TagList::new_line("Editing Repository");
                    }
                    ui.repo.handle_input(&ui.state, Key::Char(key));
                    ui.tags.handle_input(&ui.state, Key::Char(key));
                }
                Ok(Key::Backspace) => {
                    if ui.state == State::EditRepo {
                        ui.tags = tag_list::TagList::new_line("Editing Repository");
                    }
                    ui.repo.handle_input(&ui.state, Key::Backspace);
                    ui.tags.handle_input(&ui.state, Key::Backspace);
                }
                Ok(key) => {
                    ui.repo.handle_input(&ui.state, key);
                    ui.tags.handle_input(&ui.state, key);
                }
                _ => (),
            }

            //sleep for 64ms (15 fps)
            thread::sleep(std::time::Duration::from_millis(32));
        }
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
