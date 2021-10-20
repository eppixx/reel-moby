use std::{io, thread};

use crate::Opt;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

use crate::widget::info;
use crate::widget::repo_entry;
use crate::widget::service_switcher;
use crate::widget::tag_list;

pub struct Ui {
    state: State,
    repo: crate::widget::repo_entry::RepoEntry,
    tags: crate::widget::tag_list::TagList,
    services: crate::widget::service_switcher::ServiceSwitcher,
    info: crate::widget::info::Info,
}

#[derive(PartialEq, Clone)]
pub enum State {
    EditRepo,
    SelectTag,
    SelectService,
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

impl Ui {
    pub fn run(opt: &Opt) {
        let (repo_id, load_repo) = match &opt.repo {
            None => (
                "enter a repository or select one from docker-compose.yml",
                false,
            ),
            Some(repo) => (String::as_str(repo), true),
        };

        let mut ui = Ui {
            state: State::SelectService,
            repo: repo_entry::RepoEntry::new(repo_id),
            tags: tag_list::TagList::with_status("Tags are empty"),
            services: service_switcher::ServiceSwitcher::new(&opt.config).unwrap(),
            info: info::Info::new("Select image of edit Repository"),
        };

        if load_repo {
            ui.tags = tag_list::TagList::with_repo(ui.repo.get());
        }

        //setup tui
        let stdout = io::stdout().into_raw_mode().unwrap();
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        //setup input thread
        let receiver = super::spawn_stdin_channel();

        //core interaction loop
        'core: loop {
            //draw
            terminal
                .draw(|rect| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Min(9),
                                Constraint::Length(3),
                                Constraint::Min(7),
                                Constraint::Length(2),
                            ]
                            .as_ref(),
                        )
                        .split(rect.size());

                    let (list, state) = ui.services.render(ui.state == State::SelectService);
                    rect.render_stateful_widget(list, chunks[0], state);
                    rect.render_widget(ui.repo.render(ui.state == State::EditRepo), chunks[1]);
                    let (list, state) = ui.tags.render(ui.state == State::SelectTag);
                    rect.render_stateful_widget(list, chunks[2], state);
                    rect.render_widget(ui.info.render(), chunks[3]);
                })
                .unwrap();

            //handle input
            match receiver.try_recv() {
                Ok(Key::Ctrl('q')) => break 'core, //quit program without saving
                Ok(Key::Char('\t')) => {
                    ui.state.next();
                    ()
                }
                Ok(Key::Ctrl('s')) => match ui.services.save() {
                    Err(e) => {
                        ui.info.set_info(&format!("{}", e));
                        continue;
                    }
                    Ok(_) => ui.info.set_info("Saved compose file"),
                },
                Ok(Key::Ctrl('r')) => {
                    ui.repo.confirm();
                    ui.tags = tag_list::TagList::with_repo(ui.repo.get());
                }
                Ok(Key::Ctrl('n')) => match ui.tags.next_page() {
                    Err(e) => ui.info.set_info(&format!("{}", e)),
                    Ok(_) => (),
                },
                Ok(Key::Ctrl('p')) => match ui.tags.prev_page() {
                    Err(e) => ui.info.set_info(&format!("{}", e)),
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
                                ui.info.set_info(&format!("{}", e));
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
                Ok(Key::Char(key)) => match ui.state {
                    State::SelectService => (),
                    State::EditRepo => {
                        ui.info.set_info("Editing Repository");
                        ui.repo.handle_input(Key::Char(key));
                    }
                    State::SelectTag => (),
                },
                Ok(Key::Backspace) => match ui.state {
                    State::SelectService => (),
                    State::EditRepo => {
                        ui.info.set_info("Editing Repository");
                        ui.repo.handle_input(Key::Backspace);
                    }
                    State::SelectTag => (),
                },
                Ok(Key::Up) => match ui.state {
                    State::SelectService if ui.services.find_previous_match() => {
                        match ui.services.extract_repo() {
                            Err(e) => ui.info.set_info(&format!("{}", e)),
                            Ok(s) => {
                                let repo = match crate::tags::Tags::check_repo(&s) {
                                    Err(e) => {
                                        ui.info.set_info(&format!("{}", e));
                                        continue;
                                    }
                                    Ok(s) => s,
                                };
                                ui.repo.set(repo.to_string());
                                ui.tags = tag_list::TagList::with_repo(ui.repo.get());
                            }
                        }
                    }
                    State::SelectService => (),
                    State::EditRepo => (),
                    State::SelectTag => ui.tags.handle_input(Key::Up),
                },
                Ok(Key::Down) => match ui.state {
                    State::SelectService if ui.services.find_next_match() => {
                        match ui.services.extract_repo() {
                            Err(e) => ui.info.set_info(&format!("{}", e)),
                            Ok(s) => {
                                let repo = match crate::tags::Tags::check_repo(&s) {
                                    Err(e) => {
                                        ui.info.set_info(&format!("{}", e));
                                        continue;
                                    }
                                    Ok(s) => s,
                                };
                                ui.repo.set(repo.to_string());
                                ui.tags = tag_list::TagList::with_repo(ui.repo.get());
                            }
                        }
                    }
                    State::SelectService => (),
                    State::EditRepo => (),
                    State::SelectTag => ui.tags.handle_input(Key::Down),
                },
                _ => (),
            }

            //sleep for 32ms (30 fps)
            thread::sleep(std::time::Duration::from_millis(32));
        }

        terminal.clear().unwrap();
    }
}
