use anyhow::Result;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::widget::async_tag_list;
use crate::widget::info;
use crate::widget::repo_entry;
use crate::Opt;

pub struct Ui {
    state: State,
    repo: repo_entry::RepoEntry,
    tags: async_tag_list::TagList,
    details: crate::widget::details::Details,
    info: info::Info,
}

#[derive(PartialEq, Clone)]
pub enum State {
    EditRepo,
    SelectTag,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::EditRepo => write!(f, "Edit repository"),
            State::SelectTag => write!(f, "Select a tag"),
        }
    }
}

impl std::iter::Iterator for State {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            State::EditRepo => *self = State::SelectTag,
            State::SelectTag => *self = State::EditRepo,
        }
        Some(self.clone())
    }
}

pub enum UiEvent {
    NewRepo(String),
    TagInput(termion::event::Key),
    Quit,
}

impl Ui {
    #[tokio::main]
    pub async fn work_requests(ui: Arc<Mutex<Ui>>, event: std::sync::mpsc::Receiver<UiEvent>) {
        loop {
            match event.recv() {
                Ok(UiEvent::Quit) => break,
                Ok(UiEvent::NewRepo(name)) => {
                    let list = async_tag_list::TagList::with_repo_name(name).await;
                    let mut ui = ui.lock().unwrap();
                    ui.tags = list;
                }
                Ok(UiEvent::TagInput(key)) => {
                    let mut tags = {
                        let ui_data = ui.lock().unwrap();
                        ui_data.tags.clone()
                    };
                    tags.handle_input(key).await;
                    let mut ui = ui.lock().unwrap();
                    ui.tags = tags;
                }
                Err(e) => {
                    let mut ui = ui.lock().unwrap();
                    ui.info.set_info(&e);
                }
            };
        }
    }

    pub fn run(opt: &Opt) -> Result<()> {
        let repo_id = opt.repo.as_deref();

        let ui = Arc::new(Mutex::new(Ui {
            state: State::EditRepo,
            repo: repo_entry::RepoEntry::new(repo_id),
            tags: async_tag_list::TagList::with_status("no tags"),
            details: crate::widget::details::Details::new(),
            info: info::Info::new("Select image or edit Repository"),
        }));

        // spawn new thread that fetches information async
        let (sender, receiver) = std::sync::mpsc::channel();
        let ui_clone = ui.clone();
        std::thread::spawn(move || {
            Self::work_requests(ui_clone, receiver);
        });

        //setup tui
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        //setup input thread
        let receiver = super::spawn_stdin_channel();

        //core interaction loop
        'core: loop {
            let mut ui_data = ui.lock().unwrap();
            //draw
            terminal.draw(|rect| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(7),
                            Constraint::Length(2),
                        ]
                        .as_ref(),
                    )
                    .split(rect.size());
                rect.render_widget(
                    ui_data.repo.render(ui_data.state == State::EditRepo),
                    chunks[0],
                );
                let render_state = ui_data.state == State::SelectTag;
                let (tags, state) = ui_data.tags.render(render_state);
                let more_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(15), Constraint::Length(28)].as_ref())
                    .split(chunks[1]);
                rect.render_stateful_widget(tags, more_chunks[0], state);
                rect.render_widget(ui_data.details.render(), more_chunks[1]);
                rect.render_widget(ui_data.info.render(), chunks[2]);
            })?;

            //handle input
            match receiver.try_recv() {
                Ok(Key::Ctrl('q')) => {
                    sender.send(UiEvent::Quit)?;
                    break 'core; //quit program without saving
                }
                Ok(Key::Char('\t')) => {
                    ui_data.state.next();
                    let state = ui_data.state.clone();
                    ui_data.info.set_info(&state);
                }
                Ok(Key::Ctrl('r')) => {
                    ui_data.repo.confirm();
                    sender.send(UiEvent::NewRepo(ui_data.repo.get())).unwrap();
                }
                Ok(Key::Char('\n')) => match ui_data.state {
                    State::EditRepo => {
                        ui_data.repo.confirm();
                        sender.send(UiEvent::NewRepo(ui_data.repo.get())).unwrap();
                    }
                    State::SelectTag => {} // {
                                           //     let mut repo = ui_data.repo.get();
                                           //     let tag = match ui_data.tags.get_selected() {
                                           //         Err(async_tag_list::Error::NextPageSelected) => continue,
                                           //         Err(e) => {
                                           //             ui_data.info.set_info(&format!("{}", e));
                                           //             continue;
                                           //         }
                                           //         Ok(tag) => tag,
                                           //     };
                                           //     repo.push(':');
                                           //     repo.push_str(&tag);
                                           //     ui_data.services.change_current_line(repo);
                                           // }
                },
                Ok(Key::Char(key)) => match ui_data.state {
                    State::EditRepo => {
                        ui_data.info.set_text("Editing Repository");
                        ui_data.repo.handle_input(Key::Char(key));
                    }
                    State::SelectTag => {}
                },
                Ok(Key::Backspace) => match ui_data.state {
                    State::EditRepo => {
                        ui_data.info.set_text("Editing Repository");
                        ui_data.repo.handle_input(Key::Backspace);
                    }
                    State::SelectTag => {}
                },
                Ok(Key::Up) => {
                    let state = ui_data.state.clone();
                    match state {
                        State::EditRepo => {}
                        State::SelectTag => {
                            sender.send(UiEvent::TagInput(Key::Up)).unwrap();
                            ui_data.details = ui_data.tags.create_detail_widget();
                        }
                    }
                }
                Ok(Key::Down) => {
                    let state = ui_data.state.clone();
                    match state {
                        State::EditRepo => {}
                        State::SelectTag => {
                            sender.send(UiEvent::TagInput(Key::Down)).unwrap();
                            ui_data.details = ui_data.tags.create_detail_widget();
                        }
                    }
                }
                _ => {}
            }

            // release lock of ui for other threads
            drop(ui_data);

            //sleep for 32ms (30 fps)
            thread::sleep(std::time::Duration::from_millis(32));
        }

        terminal.clear()?;

        Ok(())
    }
}
