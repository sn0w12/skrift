use std::sync::mpsc::Sender;
use std::path::PathBuf;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, EventKind};

pub fn start_config_watcher(config_path: PathBuf, tx: Sender<()>) -> RecommendedWatcher {
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    let _ = tx.send(());
                }
            }
        },
        notify::Config::default(),
    ).expect("Failed to create watcher");

    watcher.watch(&config_path, RecursiveMode::NonRecursive)
        .expect("Failed to watch config file");

    watcher
}