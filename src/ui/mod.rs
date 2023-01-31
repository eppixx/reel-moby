mod no_yaml_found;
mod yaml_found;

use anyhow::Result;
use termion::input::TermRead;

use crate::widget::service_switcher;
use crate::Opt;

use std::sync::mpsc;
use std::{io, thread};

pub fn create_ui(opt: &Opt) -> Result<()> {
    let service_result = service_switcher::ServiceSwitcher::new(&opt.file);
    match service_result {
        Some(switcher) => yaml_found::Ui::run(opt, switcher),
        _ => no_yaml_found::Ui::run(opt),
    }?;

    Ok(())
}

/// create a thread for catching input and send them to core loop
pub fn spawn_stdin_channel() -> mpsc::Receiver<termion::event::Key> {
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
