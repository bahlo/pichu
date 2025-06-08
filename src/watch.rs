use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

#[derive(thiserror::Error, Debug)]
pub enum WatchError {
    #[error("Notify error: {0}")]
    Notify(#[from] notify_debouncer_mini::notify::Error),
}

/// Watch the given paths recursively and call the function on change.
///
/// # Errors
///
/// Returns an error if the watcher cannot be created, fails to watch or if a watch failed.
pub fn watch<P: AsRef<Path>>(
    paths: impl IntoIterator<Item = P>,
    on_change: impl Fn(Vec<PathBuf>),
) -> Result<(), WatchError> {
    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let mut debouncer = new_debouncer(Duration::from_millis(200), tx)?;

    for path in paths {
        debouncer.watcher().watch(
            path.as_ref(),
            notify_debouncer_mini::notify::RecursiveMode::Recursive,
        )?;
    }

    for events_res in rx {
        let changed_paths: Vec<PathBuf> = events_res?.into_iter().map(|event| event.path).collect();
        if !changed_paths.is_empty() {
            on_change(changed_paths);
        }
    }

    Ok(())
}
