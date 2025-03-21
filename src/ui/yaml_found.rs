use anyhow::Result;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use ratatui::backend::TermionBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use super::UiEvent;
use crate::error::Error;
use crate::repository;
use crate::widget::async_tag_list::{self, TagList};
use crate::widget::{info, repo_entry, service_switcher};
use crate::Args;

pub struct Ui {
    state: State,
    repo: repo_entry::RepoEntry,
    tags: TagList,
    services: service_switcher::ServiceSwitcher,
    details: crate::widget::details::Details,
    info: info::Info,
}

#[derive(PartialEq, Clone)]
pub enum State {
    EditRepo,
    SelectTag,
    SelectService,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::EditRepo => write!(f, "Edit repository"),
            State::SelectTag => write!(f, "Select a tag"),
            State::SelectService => write!(f, "Select a image"),
        }
    }
}

impl std::iter::Iterator for State {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            State::EditRepo => *self = State::SelectTag,
            State::SelectTag => *self = State::SelectService,
            State::SelectService => *self = State::EditRepo,
        }
        Some(self.clone())
    }
}

pub enum DeferredEvent {
    Quit,
    NewRepo(String),
    LoadMoreTags,
}

impl Ui {
    /// catch input and send them to core loop
    pub fn wait_for_input(sender: mpsc::Sender<UiEvent>) -> Result<(), Error> {
        let stdin = std::io::stdin();
        for c in stdin.keys() {
            sender.send(UiEvent::Input(c.unwrap()))?;
        }
        Ok(())
    }

    #[tokio::main]
    pub async fn work_requests(
        ui: &Arc<Mutex<Ui>>,
        events: mpsc::Receiver<DeferredEvent>,
        sender: mpsc::Sender<UiEvent>,
    ) -> Result<(), Error> {
        use std::sync::atomic::Ordering;
        let fetching_tags = Arc::new(std::sync::atomic::AtomicBool::new(false));
        loop {
            match events.recv() {
                Ok(DeferredEvent::Quit) => break,
                Ok(DeferredEvent::NewRepo(name)) => {
                    {
                        let mut ui = ui.lock().unwrap();
                        ui.tags = TagList::with_status("Fetching new tags...");
                        sender.send(UiEvent::RefreshOnNewData)?;
                    }
                    let list = TagList::with_repo_name(name).await;
                    let mut ui = ui.lock().unwrap();
                    ui.tags = list;
                }
                Ok(DeferredEvent::LoadMoreTags) if !fetching_tags.load(Ordering::Relaxed) => {
                    fetching_tags.store(true, Ordering::Relaxed);
                    let mut tags_copy = {
                        let mut ui = ui.lock().unwrap();
                        ui.info.set_text("Fetching more tags...");
                        sender.send(UiEvent::RefreshOnNewData)?;
                        ui.tags.clone()
                    };

                    let sender_copy = sender.clone();
                    let ui_copy = ui.clone();
                    let fetching_tags_copy = fetching_tags.clone();
                    std::thread::spawn(move || {
                        tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async {
                                tags_copy.load_next_page().await;
                                let mut ui = ui_copy.lock().unwrap();
                                //set position to the position of old TagList
                                //it may have changed since tag fetching has been invoked
                                tags_copy.set_cursor(ui.tags.get_cursor().clone());
                                ui.tags = tags_copy;
                                ui.details = ui.tags.create_detail_widget();
                                ui.info.set_text("Fetching tags done");
                                sender_copy.send(UiEvent::RefreshOnNewData).unwrap();
                                fetching_tags_copy.store(false, Ordering::Relaxed);
                            })
                    });
                }
                Ok(DeferredEvent::LoadMoreTags) => {
                    //do nothing, as we are fetching tags
                }
                Err(e) => {
                    let mut ui = ui.lock().unwrap();
                    ui.info.set_info(&e);
                }
            };
            sender.send(UiEvent::RefreshOnNewData)?;
        }
        Ok(())
    }

    pub fn run(opt: &Args, switcher: service_switcher::ServiceSwitcher) -> Result<()> {
        let repo_id = opt.repo.as_deref();

        let ui = Arc::new(Mutex::new(Ui {
            state: State::SelectService,
            repo: repo_entry::RepoEntry::new(repo_id),
            tags: TagList::with_status("no tags"),
            services: switcher,
            details: crate::widget::details::Details::new(),
            info: info::Info::new("Select image or edit Repository"),
        }));

        // spawn new thread that fetches information async
        let (sender, receiver) = mpsc::channel();
        let (deferred_sender, deferred_receiver) = mpsc::channel();
        let ui_clone = ui.clone();
        let sender2 = sender.clone();
        std::thread::spawn(move || {
            if let Err(e) = Self::work_requests(&ui_clone, deferred_receiver, sender2) {
                let mut ui = ui_clone.lock().unwrap();
                ui.info.set_info(&e);
            }
        });

        //setup tui
        let stdout = std::io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        //setup input thread
        let ui_clone = ui.clone();
        std::thread::spawn(move || {
            if let Err(e) = Self::wait_for_input(sender) {
                let mut ui = ui_clone.lock().unwrap();
                ui.info.set_info(&e);
            }
        });

        //core interaction loop
        'core: loop {
            //draw
            let mut ui_data = ui.lock().unwrap();
            terminal.draw(|rect| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(10),
                            Constraint::Min(7),
                            Constraint::Length(2),
                        ]
                        .as_ref(),
                    )
                    .split(rect.area());

                let render_state = ui_data.state == State::SelectService;
                let (file, state) = ui_data.services.render(render_state);
                rect.render_stateful_widget(file, chunks[0], state);
                let more_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Ratio(1, 3),
                            Constraint::Ratio(1, 3),
                            Constraint::Ratio(1, 3),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[1]);
                rect.render_widget(
                    ui_data.repo.render(ui_data.state == State::EditRepo),
                    more_chunks[0],
                );
                let render_state = ui_data.state == State::SelectTag;
                let (tags, state) = ui_data.tags.render(render_state);
                rect.render_stateful_widget(tags, more_chunks[1], state);
                rect.render_widget(ui_data.details.render(), more_chunks[2]);
                rect.render_widget(ui_data.info.render(), chunks[2]);
            })?;
            drop(ui_data);

            //handle events
            //wait before locking
            let event = receiver.recv();
            let mut ui_data = ui.lock().unwrap();
            match event {
                //handling input
                Ok(UiEvent::Input(key)) => match key {
                    //quit without saving
                    Key::Ctrl('q') | Key::Ctrl('c') => {
                        deferred_sender.send(DeferredEvent::Quit)?;
                        break 'core; //quit program without saving
                    }
                    //cycle widgets
                    Key::Char('\t') => {
                        ui_data.state.next();
                        let state = ui_data.state.clone();
                        ui_data.info.set_info(&state);
                    }
                    //save file
                    Key::Ctrl('s') => match ui_data.services.save() {
                        Err(e) => {
                            ui_data.info.set_info(&format!("{}", e));
                            continue;
                        }
                        Ok(_) => ui_data.info.set_text("Saved compose file"),
                    },
                    //refresh repository
                    Key::Ctrl('r') => {
                        ui_data.repo.confirm();
                        deferred_sender
                            .send(DeferredEvent::NewRepo(ui_data.repo.get()))
                            .unwrap();
                    }
                    //enter on selecting tags
                    Key::Char('\n') if ui_data.state == State::SelectTag => {
                        let mut repo = ui_data.repo.get();
                        let tag = match ui_data.tags.get_selected() {
                            Err(async_tag_list::Error::NextPageSelected) => continue,
                            Err(e) => {
                                ui_data.info.set_info(&format!("{}", e));
                                continue;
                            }
                            Ok(tag) => tag,
                        };
                        repo.push(':');
                        repo.push_str(&tag);
                        ui_data.services.change_current_line(repo);
                    }
                    //enter on editing repository
                    Key::Char('\n') if ui_data.state == State::EditRepo => {
                        ui_data.repo.confirm();
                        deferred_sender
                            .send(DeferredEvent::NewRepo(ui_data.repo.get()))
                            .unwrap();
                    }
                    //delete last char on repository
                    Key::Backspace if ui_data.state == State::EditRepo => {
                        ui_data.info.set_text("Editing Repository");
                        ui_data.repo.handle_input(Key::Backspace);
                    }
                    //moving up on selecting service
                    Key::Up | Key::Char('k')
                        if ui_data.state == State::SelectService
                            && ui_data.services.find_previous_match() =>
                    {
                        match ui_data.services.extract_repo() {
                            Err(e) => ui_data.info.set_info(&format!("{}", e)),
                            Ok(s) => {
                                let repo = match repository::check_repo(&s) {
                                    Err(e) => {
                                        ui_data.info.set_info(&format!("{}", e));
                                        continue;
                                    }
                                    Ok(s) => s,
                                };
                                ui_data.repo.set(repo.to_string());
                                deferred_sender
                                    .send(DeferredEvent::NewRepo(ui_data.repo.get()))
                                    .unwrap();
                            }
                        }
                    }
                    //moving down on selecting service
                    Key::Down | Key::Char('j')
                        if ui_data.state == State::SelectService
                            && ui_data.services.find_next_match() =>
                    {
                        match ui_data.services.extract_repo() {
                            Err(e) => ui_data.info.set_info(&format!("{}", e)),
                            Ok(s) => {
                                let repo = match repository::check_repo(&s) {
                                    Err(e) => {
                                        ui_data.info.set_info(&format!("{}", e));
                                        continue;
                                    }
                                    Ok(s) => s,
                                };
                                ui_data.repo.set(repo.to_string());
                                deferred_sender
                                    .send(DeferredEvent::NewRepo(ui_data.repo.get()))
                                    .unwrap();
                            }
                        }
                    }
                    //moving up on selecting tags
                    Key::Up | Key::Char('k') if ui_data.state == State::SelectTag => {
                        ui_data.tags.previous();
                        ui_data.details = ui_data.tags.create_detail_widget();
                    }
                    //moving down on selecting tags
                    Key::Down | Key::Char('j') if ui_data.state == State::SelectTag => {
                        if ui_data.tags.next().is_some() {
                            deferred_sender.send(DeferredEvent::LoadMoreTags).unwrap();
                        }
                        ui_data.details = ui_data.tags.create_detail_widget();
                    }
                    //append character on editing repository
                    Key::Char(key) if ui_data.state == State::EditRepo => {
                        ui_data.info.set_text("Editing Repository");
                        ui_data.repo.handle_input(Key::Char(key));
                    }
                    //ignore all else input
                    _ => {}
                },
                Ok(UiEvent::RefreshOnNewData) | Err(_) => {}
            }
        }

        terminal.clear()?;

        Ok(())
    }
}
