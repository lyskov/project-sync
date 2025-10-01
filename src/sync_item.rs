use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use colored::Colorize;
use tokio::sync::mpsc::Sender;
use tokio::time::Instant;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

pub const _SELF_CONFIG_: &str = "project-sync-self-config";
pub const _RSYNC_DEFAULT_OPTIONS_: &str = "--timeout=64 -az -e 'ssh -o ConnectTimeout=64'";

#[derive(Debug)]
pub struct SyncUpdate {
    pub name: String,
    pub update: Instant,
}

#[derive(Debug)]
pub struct SyncItem {
    pub name: String,
    pub source: String,
    pub destination: String,
    pub sync_on_start: bool,
    pub ignore: String,
    pub options: String,
    pub debounce: f32,
    pub verbose: bool,
}

impl SyncItem {
    pub fn new_config_file_sync(config_path: &Path, debounce: f32, verbose: bool) -> Self {
        SyncItem {
            name: _SELF_CONFIG_.into(),
            source: config_path.to_str().unwrap().to_string(),
            destination: config_path.to_str().unwrap().to_string(),
            sync_on_start: false,
            ignore: "".into(),
            options: "".into(),
            debounce,
            verbose,
        }
    }

    pub fn create_project_watcher(&self, transmitter: Sender<SyncUpdate>) -> notify::RecommendedWatcher {
        println!(
            // "Adding {:<16} {:>24} →     {:?} sync-on-start: {}", // {:<32}
            "Adding {} {} → {:?}", // {:<32}
            self.name.bright_green(),
            self.source,
            self.destination,
            //self.sync_on_start
        );

        let path = PathBuf::from(shellexpand::tilde(&self.source).as_ref());

        //println!("Creating watcher for project {} in directory {}", name.green(), path.display());

        let project_name = self.name.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(_) => transmitter
                .blocking_send(SyncUpdate {
                    name: project_name.clone(),
                    update: Instant::now(),
                })
                .unwrap(),
            Err(e) => println!("Watcher for project {} got Error {:?}", project_name.clone(), e),
        })
        .unwrap();
        notify::Watcher::watch(&mut watcher, &path, notify::RecursiveMode::Recursive).unwrap();
        watcher
    }

    fn rsync_extras(&self) -> String {
        let rsync_ignore_file_path_root = PathBuf::from(shellexpand::tilde("~/.cache/project-sync").as_ref());
        std::fs::create_dir_all(&rsync_ignore_file_path_root).unwrap();
        let rsync_ignore_path = rsync_ignore_file_path_root.join(self.name.clone() + ".rsync-ignore");
        std::fs::write(&rsync_ignore_path, &self.ignore).expect("Can not create rsync-ignore file...");

        let options = if self.options.is_empty() { "" } else { &format!("{} ", self.options) };

        format!("{}--exclude-from='{}'", options, rsync_ignore_path.display())
    }

    async fn run_rsync(&self, previous_sync_time: Instant) -> Instant {
        let extras = self.rsync_extras();
        let verbosity = if self.verbose { "--itemize-changes " } else { "" };
        let command_line = format!("rsync {verbosity}{_RSYNC_DEFAULT_OPTIONS_} {extras} {} '{}'", self.source, self.destination);

        let synced = Instant::now();

        println!(
            "{} Syncing {} {}",
            format!("[{}]", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).bright_blue(),
            self.name.bright_green(),
            command_line.white().dimmed()
        );

        let mut attempts = 1;
        while !tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&command_line)
            .status()
            .await
            .expect("could not run shell command...")
            .success()
        {
            println!("{} to sync {} → {}... going to sleep and retry...", "[FAILED]".on_red(), self.name, self.destination);
            std::thread::sleep(Duration::from_secs(4 + attempts * 2));
            if attempts > 1 {
                println!("Too many failed attempts: {}, giving up...", attempts.to_string().bright_red());
                return previous_sync_time;
            }
            attempts += 1;
        }
        //sleep(Duration::from_secs_f32(10.25));
        println!(
            "{} Synced {} {}",
            format!("[{}]", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).bright_blue(),
            self.name.bright_green(),
            command_line.white().dimmed()
        );
        synced
    }

    async fn process_update_event(&self, cancellation_token: &CancellationToken, SyncUpdate { name, update }: SyncUpdate, mut synced: Instant) -> Instant {
        // println!("{project_update:?}");
        if name == _SELF_CONFIG_ {
            println!(
                "{} file {} detected, reloading...\n",
                "UPDATE to configuration".truecolor(239, 134, 62).underline(),
                self.source.green()
            );
            cancellation_token.cancel();
            return synced;
        }

        if update >= synced {
            let duration = update.duration_since(synced);
            if duration.as_secs_f32() < self.debounce {
                sleep(Duration::from_secs_f32(self.debounce) - duration).await;
            }
            synced = self.run_rsync(synced).await;
        }
        synced
    }

    pub async fn sync(&self, cancellation_token: CancellationToken) {
        let mut synced = Instant::now();

        let (tx, mut rx) = tokio::sync::mpsc::channel::<SyncUpdate>(1024);

        let _watcher = self.create_project_watcher(tx.clone());

        if self.sync_on_start {
            //println!("Scheduling initial sync for {}...", name.green());
            tx.send(SyncUpdate {
                name: self.name.clone(),
                update: Instant::now(),
            })
            .await
            .unwrap();
        }

        loop {
            tokio::select! {
                maybe_update = rx.recv() => {
                    if let Some(project_update) = maybe_update {
                        synced = self.process_update_event(&cancellation_token, project_update, synced).await;
                    }
                    else {
                        break; // channel closed
                    }
                }
                _ = cancellation_token.cancelled() => {
                    // println!("Cancellation requested, terminating...");
                    break;
                }
            }
        }
    }
}
