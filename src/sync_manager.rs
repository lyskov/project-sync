use colored::Colorize;
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::sync_item::SyncItem;

pub struct Projects(Vec<SyncItem>);

impl Projects {
    fn new(config: Config) -> Self {
        println!("Reading sync config from {}...", config.config_path.display().to_string().bright_yellow());

        let mut p: Vec<SyncItem> = config
            .sync
            .into_iter()
            .flat_map(|c| {
                c.destinations.into_iter().map(move |d| SyncItem {
                    name: c.name.clone(),
                    source: c.source.clone(),
                    destination: d.clone(),
                    sync_on_start: c.sync_on_start,
                    ignore: c.ignore.clone(),
                    options: c.options.clone(),
                    debounce: config.debounce,
                    verbose: config.verbose,
                })
            })
            .collect();

        let config_sync = SyncItem::new_config_file_sync(&config.config_path, config.debounce, config.verbose);
        p.push(config_sync);

        Projects(p)
    }
    pub async fn sync_projects(&self) {
        let cancellation_token = CancellationToken::new();
        futures::future::join_all(self.0.iter().map(|p| async { p.sync(cancellation_token.clone()).await })).await;
    }
}

pub async fn sync_projects(config: Config) {
    let projects_watch = Projects::new(config);
    projects_watch.sync_projects().await;
}
