use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use chrono::{DateTime, Datelike, Utc};
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use o2o::o2o;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};

#[derive(Deserialize, Serialize, Debug)]
pub struct Messages {
    pub motds: Vec<Arc<str>>,
    pub likes: Arc<str>,
    pub working_on: Arc<str>,
    pub current_motd: usize,
    pub last_motd_update: DateTime<Utc>
}

pub fn parse_toml<T: for<'de> Deserialize<'de>>(
    path: &'static str
) -> anyhow::Result<T> {
    toml::from_str(&fs::read_to_string(path)?).map_err(|e| e.into())
}

pub fn write_toml<T: Serialize>(
    path: &'static str,
    data: &T
) -> anyhow::Result<()> {
    if fs::exists(path).unwrap_or(true) {
        fs::create_dir_all("./backups/")?;
        let mut backup_path: PathBuf = "./backups/".parse()?;
        backup_path.push(format!("{}.bak-{}", path, Utc::now().timestamp_millis()));
        fs::copy(path, backup_path)?;
    }
    
    Ok(fs::write(path, toml::to_string_pretty(data)?)?)
}

pub static MESSAGES: LazyLock<RwLock<Messages>> = LazyLock::new(|| RwLock::new(parse_toml("./messages.toml").expect("could not load messages.toml")));

fn watch_event(event: notify::Result<Event>) {
    let Ok(event) = event else { return };
    
    if matches!(event.kind, EventKind::Modify(..) | EventKind::Create(..)) {
        let Ok(new_messages) = parse_toml("./messages.toml") else { return; };
        
        *MESSAGES.blocking_write() = new_messages;
    }
}

pub fn start_watching() {
    let mut watcher = Box::new(recommended_watcher(watch_event).expect("could not create messages.toml watcher"));
    watcher.watch("./messages.toml".as_ref(), RecursiveMode::NonRecursive).expect("could not start watching ./messages.toml");
    Box::leak(watcher); // leaks memory 😎
}

pub async fn start_motd_timer() {
    tokio::spawn(async {
        loop {
            {
                let mut messages = MESSAGES.write().await;

                if Utc::now().num_days_from_ce() != messages.last_motd_update.num_days_from_ce()
                    && messages.current_motd != messages.motds.len() - 1 {
                    messages.current_motd += 1;
                    messages.last_motd_update = Utc::now();
                    if let Err(e) = write_toml("messages.toml", &*messages) {
                        println!("motd timer write_toml error: {e}");
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(60 * 5)).await;
        }
    });
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub admin_password: Box<str>,
    pub port: u16
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| parse_toml("./config.toml").expect("could not load config.toml"));
