use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

#[derive(thiserror::Error, Debug)]
pub enum WatchError {
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
}

/// Watch the given directory recursively and call the function on change.
///
/// # Errors
///
/// Returns an error if a the watcher cannot be created, fails to watch or if a watch failed.
pub fn watch(path: impl AsRef<Path>, on_change: impl Fn(Vec<PathBuf>)) -> Result<(), WatchError> {
    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        let event = res?;
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                on_change(event.paths);
            }
            _ => {} // ignore
        }
    }

    Ok(())
}
