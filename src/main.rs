use std::path::PathBuf;
use structopt::StructOpt;

mod repo;
mod tags;
mod ui;
mod widget;

/// helps you searching or updating tags of your used docker images
#[derive(StructOpt, Debug)]
#[structopt(name = "main")]
pub struct Opt {
    /// Show architectures of images and their sizes
    #[structopt(short, long)]
    verbose: bool,

    /// A custom path to a docker-compose file
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// Give a Repository identifier, e.g. library/nginx
    #[structopt(short, long, parse(from_str))]
    repo: Option<String>,
}

fn main() {
    //parse parameter
    let opt = Opt::from_args();
    ui::Ui::run(&opt);
}
