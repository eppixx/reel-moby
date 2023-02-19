use anyhow::Result;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crate::widget::async_tag_list::{self, TagList};
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
    Input(Key),
    RefreshOnNewData,
}

pub enum DeferredEvent {
    Quit,
    NewRepo(String),
    TagPrevious,
    TagNext,
}

impl Ui {
    /// create a thread for catching input and send them to core loop
    pub fn spawn_input_channel(sender: mpsc::Sender<UiEvent>) {
        thread::spawn(move || loop {
            let stdin = io::stdin();
            for c in stdin.keys() {
                sender.send(UiEvent::Input(c.unwrap())).unwrap();
            }
        });
    }

    #[tokio::main]
    pub async fn work_requests(
        ui: Arc<Mutex<Ui>>,
        events: mpsc::Receiver<DeferredEvent>,
        sender: mpsc::Sender<UiEvent>,
    ) {
        loop {
            match events.recv() {
                Ok(DeferredEvent::Quit) => break,
                Ok(DeferredEvent::NewRepo(name)) => {
                    {
                        let mut ui = ui.lock().unwrap();
                        ui.tags = TagList::with_status("fetching new tags...");
                        sender.send(UiEvent::RefreshOnNewData).unwrap();
                    }
                    let list = async_tag_list::TagList::with_repo_name(name).await;
                    let mut ui = ui.lock().unwrap();
                    ui.tags = list;
                }
                Ok(DeferredEvent::TagPrevious) => {
                    let mut ui = ui.lock().unwrap();
                    ui.tags.previous();
                    ui.details = ui.tags.create_detail_widget();
                }
                Ok(DeferredEvent::TagNext) => {
                    let (fetched_new_tags, mut tags) = {
                        let mut ui = ui.lock().unwrap();
                        if ui.tags.at_end_of_list() {
                            ui.info.set_text("Fetching more tags...");
                            sender.send(UiEvent::RefreshOnNewData).unwrap();
                            (true, ui.tags.clone())
                        } else {
                            (false, ui.tags.clone())
                        }
                    };
                    tags.next().await;
                    let mut ui = ui.lock().unwrap();
                    ui.tags = tags;
                    ui.details = ui.tags.create_detail_widget();
                    if fetched_new_tags {
                        ui.info.set_text("Fetching tags done");
                    }
                }
                Err(e) => {
                    let mut ui = ui.lock().unwrap();
                    ui.info.set_info(&e);
                }
            };
            sender.send(UiEvent::RefreshOnNewData).unwrap();
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
        let (sender, receiver) = mpsc::channel();
        let (deferred_sender, deferred_receiver) = mpsc::channel();
        let ui_clone = ui.clone();
        let sender2 = sender.clone();
        std::thread::spawn(move || {
            Self::work_requests(ui_clone, deferred_receiver, sender2);
        });

        //setup tui
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        //setup input thread
        Self::spawn_input_channel(sender);

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
            drop(ui_data);

            //handle input
            //wait before locking
            let event = receiver.recv();
            let mut ui_data = ui.lock().unwrap();
            match event {
                Ok(UiEvent::Input(Key::Ctrl('q'))) | Ok(UiEvent::Input(Key::Ctrl('c'))) => {
                    deferred_sender.send(DeferredEvent::Quit)?;
                    break 'core; //quit program without saving
                }
                Ok(UiEvent::Input(Key::Char('\t'))) => {
                    ui_data.state.next();
                    let state = ui_data.state.clone();
                    ui_data.info.set_info(&state);
                }
                Ok(UiEvent::Input(Key::Ctrl('r'))) => {
                    ui_data.repo.confirm();
                    deferred_sender
                        .send(DeferredEvent::NewRepo(ui_data.repo.get()))
                        .unwrap();
                }
                Ok(UiEvent::Input(Key::Char('\n'))) if ui_data.state == State::EditRepo => {
                    ui_data.repo.confirm();
                    deferred_sender
                        .send(DeferredEvent::NewRepo(ui_data.repo.get()))
                        .unwrap();
                }
                Ok(UiEvent::Input(Key::Backspace)) if ui_data.state == State::EditRepo => {
                    ui_data.info.set_text("Editing Repository");
                    ui_data.repo.handle_input(Key::Backspace);
                }
                Ok(UiEvent::Input(Key::Up)) | Ok(UiEvent::Input(Key::Char('k')))
                    if ui_data.state == State::SelectTag =>
                {
                    deferred_sender.send(DeferredEvent::TagPrevious).unwrap();
                    ui_data.details = ui_data.tags.create_detail_widget();
                }
                Ok(UiEvent::Input(Key::Down)) | Ok(UiEvent::Input(Key::Char('j')))
                    if ui_data.state == State::SelectTag =>
                {
                    deferred_sender.send(DeferredEvent::TagNext).unwrap();
                    ui_data.details = ui_data.tags.create_detail_widget();
                }
                Ok(UiEvent::Input(Key::Char(key))) if ui_data.state == State::EditRepo => {
                    ui_data.info.set_text("Editing Repository");
                    ui_data.repo.handle_input(Key::Char(key));
                }
                _ => {}
            }
        }

        terminal.clear()?;

        Ok(())
    }
}
