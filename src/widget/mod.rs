use tui::widgets::ListState;

pub mod repo_entry;
pub mod tag_list;

pub trait Widget {
    fn input(&mut self, event: termion::event::Key);
}
