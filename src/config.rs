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

pub fn load_messages() -> anyhow::Result<Messages> {
    dbg!(toml::from_str(&fs::read_to_string("./messages.toml")?).map_err(|e| e.into()))
}

pub static MESSAGES: LazyLock<RwLock<Messages>> = LazyLock::new(|| RwLock::new(load_messages().expect("could not load messages.toml")));

fn watch_event(event: notify::Result<Event>) {
    let Ok(event) = event else { return };
    
    if matches!(event.kind, EventKind::Modify(..) | EventKind::Create(..)) {
        *MESSAGES.blocking_write() = load_messages().expect("could not update messages.toml");
    }
}

pub fn start_watching() {
    let mut watcher = Box::new(recommended_watcher(watch_event).expect("could not create messages.toml watcher"));
    watcher.watch("./messages.toml".as_ref(), RecursiveMode::NonRecursive).expect("could not start watching ./messages.toml");
    Box::leak(watcher);
}