use std::{io, thread};

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

use crate::widget::details;
use crate::widget::info;
use crate::widget::repo_entry;
use crate::widget::tag_list;
use crate::Opt;

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

pub struct NoYaml {
    state: State,
    repo: repo_entry::RepoEntry,
    tags: tag_list::TagList,
    details: details::Details,
    info: info::Info,
}

impl NoYaml {
    pub fn run(opt: &Opt) {
        let repo_id = opt.repo.as_deref();

        let mut ui = NoYaml {
            state: State::EditRepo,
            repo: repo_entry::RepoEntry::new(repo_id),
            tags: tag_list::TagList::with_status("Tags are empty"),
            details: details::Details::new(),
            info: info::Info::new("could not find a docker-compose file"),
        };

        // load tags if a repository was given thorugh paramter
        if opt.repo.is_none() {
            ui.tags = tag_list::TagList::with_repo_name(ui.repo.get());
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
                                Constraint::Length(3),
                                Constraint::Min(7),
                                Constraint::Length(2),
                            ]
                            .as_ref(),
                        )
                        .split(rect.size());

                    rect.render_widget(ui.repo.render(ui.state == State::EditRepo), chunks[0]);
                    let (list, state) = ui.tags.render(ui.state == State::SelectTag);
                    let more_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(15), Constraint::Length(28)].as_ref())
                        .split(chunks[1]);
                    rect.render_stateful_widget(list, more_chunks[0], state);
                    rect.render_widget(ui.details.render(), more_chunks[1]);
                    rect.render_widget(ui.info.render(), chunks[2]);
                })
                .unwrap();

            //handle input
            match receiver.try_recv() {
                Ok(Key::Ctrl('q')) => break 'core,
                Ok(Key::Char('\t')) => {
                    ui.state.next();
                    ui.info.set_info(&ui.state);
                }
                Ok(Key::Ctrl('r')) => {
                    ui.repo.confirm();
                    ui.tags = tag_list::TagList::with_repo_name(ui.repo.get());
                }
                Ok(Key::Char('\n')) => match ui.state {
                    State::EditRepo => {
                        ui.repo.confirm();
                        ui.tags = tag_list::TagList::with_repo_name(ui.repo.get());
                    }
                    State::SelectTag => ui.tags.handle_input(Key::Char('\n')),
                },
                Ok(Key::Char(key)) => match ui.state {
                    State::EditRepo => {
                        ui.info.set_text("Editing Repository");
                        ui.repo.handle_input(Key::Char(key));
                    }
                    State::SelectTag => {
                        ui.tags.handle_input(Key::Char(key));
                    }
                },
                Ok(Key::Backspace) => match ui.state {
                    State::EditRepo => {
                        ui.info.set_text("Editing Repository");
                        ui.repo.handle_input(Key::Backspace);
                    }
                    State::SelectTag => (),
                },
                Ok(Key::Up) => match ui.state {
                    State::EditRepo => (),
                    State::SelectTag => {
                        ui.tags.handle_input(Key::Up);
                        ui.details = ui.tags.create_detail_widget();
                    }
                },
                Ok(Key::Down) => match ui.state {
                    State::EditRepo => (),
                    State::SelectTag => {
                        ui.tags.handle_input(Key::Down);
                        ui.details = ui.tags.create_detail_widget();
                    }
                },
                _ => (),
            }

            //sleep for 32ms (30 fps)
            thread::sleep(std::time::Duration::from_millis(32));
        }

        terminal.clear().unwrap();
    }
}
