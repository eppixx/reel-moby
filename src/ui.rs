use std::sync::mpsc;
use std::{io, thread};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

use crate::widget::repo_entry;
use crate::widget::service_switcher;
use crate::widget::tag_list;

pub struct Ui {
    state: State,
    repo: crate::widget::repo_entry::RepoEntry,
    tags: crate::widget::tag_list::TagList,
    services: crate::widget::service_switcher::ServiceSwitcher,
}

#[derive(PartialEq, Clone)]
pub enum State {
    EditRepo,
    SelectTag,
    SelectService,
}

impl State {
    fn next(&self) -> State {
        match self {
            State::EditRepo => State::SelectTag,
            State::SelectTag => State::SelectService,
            State::SelectService => State::EditRepo,
        }
    }
}

impl Ui {
    pub fn run(repo_id: &str) {
        let mut ui = Ui {
            state: State::SelectService,
            repo: repo_entry::RepoEntry::new(repo_id),
            tags: tag_list::TagList::with_status("Select image or edit Repository"),
            services: service_switcher::ServiceSwitcher::new(),
        };

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
                        .constraints(
                            [
                                Constraint::Min(3),
                                Constraint::Length(3),
                                Constraint::Max(7),
                            ]
                            .as_ref(),
                        )
                        .split(rect.size());

                    let (list, state) = ui.services.render(&ui.state);
                    rect.render_stateful_widget(list, chunks[0], state);
                    rect.render_widget(ui.repo.render(&ui.state), chunks[1]);
                    let (list, state) = ui.tags.render(&ui.state);
                    rect.render_stateful_widget(list, chunks[2], state);
                })
                .unwrap();

            //handle input
            match receiver.try_recv() {
                Ok(Key::Ctrl('q')) => break 'core, //quit program without saving
                Ok(Key::Char('\t')) => ui.state = ui.state.next(),
                Ok(Key::Ctrl('s')) => match ui.services.save() {
                    Err(e) => {
                        ui.show_info(&format!("{}", e));
                        continue;
                    }
                    Ok(_) => (),
                },
                Ok(Key::Char('\n')) => match ui.state {
                    State::EditRepo => {
                        ui.repo.confirm();
                        ui.tags = tag_list::TagList::with_repo(ui.repo.get());
                    }
                    State::SelectTag => {
                        let mut repo = ui.repo.get();
                        let tag = match ui.tags.get_selected() {
                            Err(e) => {
                                ui.show_info(&format!("{}", e));
                                continue;
                            }
                            Ok(tag) => tag,
                        };
                        repo.push_str(":");
                        repo.push_str(&tag);
                        ui.services.change_current_line(repo);
                    }
                    _ => (),
                },
                Ok(Key::Char(key)) => {
                    if ui.state == State::EditRepo {
                        ui.show_info("Editing Repository");
                    }
                    ui.repo.handle_input(&ui.state, Key::Char(key));
                    ui.tags.handle_input(&ui.state, Key::Char(key));
                }
                Ok(Key::Backspace) => {
                    if ui.state == State::EditRepo {
                        ui.show_info("Editing Repository");
                    }
                    ui.repo.handle_input(&ui.state, Key::Backspace);
                    ui.tags.handle_input(&ui.state, Key::Backspace);
                }
                Ok(Key::Up) => {
                    if ui.state == State::SelectService && ui.services.find_previous_match() {
                        match ui.services.extract_repo() {
                            Err(e) => ui.show_info(&format!("{}", e)),
                            Ok(s) => {
                                let repo = match crate::tags::Tags::check_repo(s) {
                                    Err(e) => {
                                        ui.show_info(&format!("{}", e));
                                        continue;
                                    }
                                    Ok(s) => s,
                                };
                                ui.repo.set(repo);
                                ui.tags = tag_list::TagList::with_repo(ui.repo.get());
                            }
                        }
                    }
                    ui.tags.handle_input(&ui.state, Key::Up);
                    ui.repo.handle_input(&ui.state, Key::Up);
                }
                Ok(Key::Down) => match ui.state {
                    State::SelectService if ui.services.find_next_match() => {
                        match ui.services.extract_repo() {
                            Err(e) => ui.show_info(&format!("{}", e)),
                            Ok(s) => {
                                let repo = match crate::tags::Tags::check_repo(s) {
                                    Err(e) => {
                                        ui.show_info(&format!("{}", e));
                                        continue;
                                    }
                                    Ok(s) => s,
                                };
                                ui.repo.set(repo);
                                ui.tags = tag_list::TagList::with_repo(ui.repo.get());
                            }
                        }
                    }
                    _ => {
                        ui.tags.handle_input(&ui.state, Key::Down);
                        ui.repo.handle_input(&ui.state, Key::Down);
                    }
                },
                Ok(key) => {
                    ui.repo.handle_input(&ui.state, Key::Down);
                    ui.tags.handle_input(&ui.state, key);
                }
                _ => (),
            }

            //sleep for 32ms (30 fps)
            thread::sleep(std::time::Duration::from_millis(32));
        }

        terminal.clear().unwrap();
    }

    /// helper function to show information in TagList
    fn show_info(&mut self, error: &str) {
        self.tags = tag_list::TagList::with_status(error);
    }

    /// create a thread for catching input and send them to core loop
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
