//! FS changes' notifier.
//!
//! Simple library to watch file changes inside given directory.
//!
//! Usage example:
//!
//! ```rust,ignore
//! use fs_change_notifier::{create_watcher, match_event, RecursiveMode};
//!
//! let root = PathBuf::from(".");
//! let (mut wr, rx) = create_watcher(|e| log::error!("{e:?}")).unwrap();
//! wr.watch(&root, RecursiveMode::Recursive).unwrap();
//!
//! loop {
//!     tokio::select! {
//!         _ = your_job => {},
//!         _ = match_event(&root, rx, &exclude) => {
//!             // do your logic on fs update
//!         },
//!     }
//! }
//! ```

#![deny(warnings, missing_docs, clippy::todo, clippy::unimplemented)]

use notify::event::{CreateKind, ModifyKind};
use notify::{Event, EventKind, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

pub use notify::RecursiveMode;

/// Creates a watcher and an associated MPSC channel receiver.
pub fn create_watcher(
    err_handler: impl Fn(notify::Error) + Send + 'static,
) -> anyhow::Result<(Box<dyn Watcher>, mpsc::Receiver<Event>)> {
    let (tx, rx) = mpsc::channel::<Event>(1000);
    let mut watcher = notify::recommended_watcher(move |ev: notify::Result<Event>| match ev {
        Ok(ev) => {
            let _ = tx.blocking_send(ev);
        }
        Err(e) => err_handler(e),
    })?;

    watcher.configure(notify::Config::default().with_follow_symlinks(false))?;

    Ok((Box::new(watcher) as Box<dyn Watcher>, rx))
}

/// Matches event and returns on included ones.
///
/// This function relies on given `root` and `exclude` set to watch changes happened
/// only inside given directory and not with excluded files.
pub async fn match_event(root: &Path, mut rx: mpsc::Receiver<Event>, exclude: &HashSet<PathBuf>) {
    loop {
        let event = if let Some(ev) = rx.recv().await {
            ev
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        };

        let has_non_excluded = event.paths.iter().any(|event_path| {
            !exclude.iter().any(|exclude_path| {
                let event = event_path.to_string_lossy();
                let exclude = exclude_path.to_string_lossy();

                if exclude.contains('*') {
                    let parts = exclude.split('*').collect::<Vec<_>>();
                    if parts.len() != 2 {
                        false
                    } else if let Ok(relative_event) = event_path.strip_prefix(root).map(|p| p.to_string_lossy()) {
                        relative_event.starts_with(parts[0]) && relative_event.ends_with(parts[1])
                    } else {
                        event.contains(parts[0]) && event.ends_with(parts[1])
                    }
                } else {
                    event.contains(exclude.as_ref())
                }
            })
        });

        if has_non_excluded {
            match event.kind {
                EventKind::Create(CreateKind::File) => return,
                EventKind::Modify(ModifyKind::Name(_)) | EventKind::Modify(ModifyKind::Data(_)) => return,
                EventKind::Remove(_) => return,
                _ => {}
            }
        }
    }
}
