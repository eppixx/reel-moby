use std::path::PathBuf;
use clap::Parser;

mod repo;
mod repository;
mod ui;
mod widget;

/// helps you searching or updating tags of your used docker images
#[derive(Parser, Debug)]
pub struct Args {
    /// A custom path to a docker-compose file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Give a Repository identifier, e.g. library/nginx
    #[arg(short, long)]
    repo: Option<String>,
}

fn main() {
    //parse parameter
    let args = Args::parse();
    ui::create_ui(&args);
}
