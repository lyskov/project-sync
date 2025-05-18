use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, Instant},
};

use colored::Colorize;

use crate::config;

const _SELF_CONFIG_: &str = "project-sync-self-config";

#[derive(Debug)]
//#[allow(dead_code)]
pub struct Sync {
    pub name: String,
    pub source: String,
    pub destination: String,
    pub synced: Instant,
    //updated: Instant,
    pub sync_on_start: bool,
    pub ignore: String,
    pub options: String,
}

#[derive(Debug)]
struct SyncUpdate {
    name: String,
    update: Instant,
}

impl Sync {
    fn sync_extras(&self) -> String {
        // let git_ignore = ".git\n";
        // let rsync_ignore = self.ignore.as_deref().unwrap_or(git_ignore);
        // let rsync_ignore = if rsync_ignore.starts_with(git_ignore) {
        //     git_ignore
        // } else {
        //     &format!("{}{}", git_ignore, rsync_ignore)
        // };

        let rsync_ignore_dir = PathBuf::from(shellexpand::tilde("~/.cache/project-sync").as_ref());
        std::fs::create_dir_all(&rsync_ignore_dir).unwrap();

        let rsync_ignore_path = rsync_ignore_dir.join(self.name.clone() + ".rsync-ignore");

        std::fs::write(&rsync_ignore_path, &self.ignore).expect("Can not create rsync-ignore file...");

        format!("{} --exclude-from='{}'", self.options, rsync_ignore_path.display())
    }

    fn sync(&mut self) {
        //std::thread::sleep(Duration::from_secs_f32(0.25));
        print!("Syncing {} ", self.name.bright_green());

        let verbosity = "--itemize-changes"; // -v
        let extras = self.sync_extras();
        let command_line = format!("rsync {verbosity} -az -e ssh {extras} {} '{}'", self.source, self.destination);

        println!("{}", command_line.white().dimmed());

        let mut attempts = 0;
        let start = Instant::now();

        self.synced = Instant::now();
        while !std::process::Command::new("sh")
            .arg("-c")
            .arg(&command_line)
            .status()
            .expect("could not run shell command...")
            .success()
        {
            println!("{}... going to sleep and retry...", "FAILED".on_red());
            std::thread::sleep(Duration::from_secs(4));
            if attempts > 7 {
                println!("Too many failed attempts: {}, giving up...", attempts.to_string().bright_red());
                break;
            }
            attempts += 1;
        }

        let _duration = start.elapsed();
        //println!("Elapsed time: {:.2?}", _duration);
    }
    fn create_project_watcher(&self, transmitter: std::sync::mpsc::Sender<SyncUpdate>) -> notify::RecommendedWatcher {
        println!(
            "Adding {:<16} {:>24} → {:<32} sync-on-start: {}",
            self.name.bright_green(),
            self.source,
            self.destination,
            self.sync_on_start
        );

        let path = PathBuf::from(shellexpand::tilde(&self.source).as_ref());

        //println!("Creating watcher for project {} in directory {}", name.green(), path.display());

        let project_name = self.name.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(_) => transmitter
                .send(SyncUpdate {
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
}

struct ProjectConfig {
    projects: BTreeMap<String, Sync>,
    debounce: f32,
    filter: Option<String>,
}

impl ProjectConfig {
    fn sync_projects(&mut self) {
        // Retain only those entries that match the filter string
        if let Some(filter) = &self.filter {
            self.projects.retain(|_name, project| {
                project.destination.contains(filter) //|| project.path.contains(&filter)
            });
        }

        let (tx, rx) = std::sync::mpsc::channel::<SyncUpdate>();

        let mut watchers = Vec::<_>::new();
        for (name, p) in &self.projects {
            watchers.push(p.create_project_watcher(tx.clone()));

            if p.sync_on_start {
                //println!("Scheduling initial sync for {}...", name.green());
                tx.send(SyncUpdate {
                    name: name.clone(),
                    update: Instant::now(),
                })
                .unwrap();
            }
        }
        println!();

        while let Ok(project_update) = rx.recv() {
            // println!("{project_update:?}");

            let p = self.projects.get_mut(&project_update.name).unwrap();

            if project_update.name == _SELF_CONFIG_ {
                println!(
                    "{} file {} detected, reloading...\n",
                    "UPDATE to configuration".truecolor(239, 134, 62).underline(),
                    p.source.green()
                );
                return;
            }

            if project_update.update > p.synced {
                let duration = project_update.update.duration_since(p.synced);
                if duration.as_secs_f32() < self.debounce {
                    sleep(Duration::from_secs_f32(self.debounce) - duration);
                }
                p.sync();
            }
        }
    }
}

fn run_with_project_config_factory<F>(project_config_factory: F, config_path: &Path)
where
    F: Fn() -> ProjectConfig,
{
    loop {
        let mut project_config: ProjectConfig = project_config_factory();

        project_config.projects.insert(
            _SELF_CONFIG_.into(),
            Sync {
                name: _SELF_CONFIG_.into(),
                source: config_path.to_str().unwrap().to_string(),
                destination: config_path.to_str().unwrap().to_string(),
                synced: Instant::now(),
                sync_on_start: false,
                ignore: "".into(),
                options: "".into(),
            },
        );

        project_config.sync_projects();
    }
}

pub fn run(config_path: &Path, filter: Option<String>) {
    let projects_factory = move || -> ProjectConfig {
        println!("Reading sync config from {}...", config_path.display().to_string().bright_yellow());
        let config = config::read_config(config_path);
        // println!("Parsed config: {:#?}", config);
        let projects = config
            .sync
            .into_iter()
            .map(|c| {
                (
                    c.name.clone(),
                    Sync {
                        name: c.name,
                        source: c.source,
                        destination: c.destination,
                        synced: Instant::now(),
                        sync_on_start: c.sync_on_start,
                        ignore: c.ignore,
                        options: c.options,
                    },
                )
            })
            .collect();

        ProjectConfig {
            projects,
            debounce: config.debounce,
            filter: filter.clone(),
        }
    };

    run_with_project_config_factory(projects_factory, config_path);
}
