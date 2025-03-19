use std::path::PathBuf;
use structopt::StructOpt;

mod repo;
mod repository;
mod ui;
mod widget;

/// helps you searching or updating tags of your used docker images
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// A custom path to a docker-compose file
    #[structopt(short, long, parse(from_os_str))]
    file: Option<PathBuf>,

    /// Give a Repository identifier, e.g. library/nginx
    #[structopt(short, long, parse(from_str))]
    repo: Option<String>,
}

fn main() {
    //parse parameter
    let opt = Opt::from_args();
    ui::create_ui(&opt);
}
