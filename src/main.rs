use std::path::PathBuf;

use clap::Parser;

use crate::config::Config;

mod config;
mod projects_watcher;
mod sync;

#[derive(Parser, Debug)]
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    loop {
        let mut config = Config::from_config_path(&args.config);

        if let Some(filter) = &args.filter {
            config.retain_destinations(filter);
        }

        config.verbose = args.verbose;
        // if let Some(verbose) = args.verbose {
        //     config.verbose = verbose;
        // }

        let projects_watch = projects_watcher::ProjectsWatch::new(config);
        projects_watch.sync_projects().await;
    }
}
