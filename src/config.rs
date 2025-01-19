use std::fs;
use std::sync::{Arc, LazyLock};
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use o2o::o2o;
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};

#[derive(Deserialize, Debug)]
pub struct Messages {
    pub motds: Vec<Arc<str>>,
    pub likes: Arc<str>,
    pub working_on: Arc<str>
}

pub fn parse_toml<T: std::fmt::Debug + for<'de> Deserialize<'de>>(
    path: &'static str
) -> anyhow::Result<T> {
    dbg!(toml::from_str(&fs::read_to_string(path)?).map_err(|e| e.into()))
}

pub static MESSAGES: LazyLock<RwLock<Messages>> = LazyLock::new(|| RwLock::new(parse_toml("./messages.toml").expect("could not load messages.toml")));

fn watch_event(event: notify::Result<Event>) {
    let Ok(event) = event else { return };
    
    if matches!(event.kind, EventKind::Modify(..) | EventKind::Create(..)) {
        *MESSAGES.blocking_write() = parse_toml("./messages.toml").expect("could not update messages.toml");
    }
}

pub fn start_watching() {
    let mut watcher = Box::new(recommended_watcher(watch_event).expect("could not create messages.toml watcher"));
    watcher.watch("./messages.toml".as_ref(), RecursiveMode::NonRecursive).expect("could not start watching ./messages.toml");
    Box::leak(watcher);
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub admin_password: Box<str>
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| parse_toml("./config.toml").expect("could not load config.toml"));
