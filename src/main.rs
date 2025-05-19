use std::path::PathBuf;

use clap::Parser as clap_Parser;

mod config;
mod sync;

#[derive(clap_Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Activate debug mode
    #[arg(short, long)]
    debug: bool,

    /// Show more logging details
    #[arg(short, long)]
    verbose: bool,

    /// Filter configured project list
    filter: Option<String>,

    /// Path TOML file with sync config
    #[arg(short, long)]
    config: PathBuf,
}

fn main() {
    let args = Args::parse();

    let config_path = args.config;
    sync::run(&config_path, args.filter);
}
