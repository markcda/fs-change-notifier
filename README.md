# fs-change-notifier

Simple library to watch file changes inside given directory.

Usage example:

```rust
use fs_change_notifier::{create_watcher, RecursiveMode};

let (mut wr, rx) = create_watcher(|e| log::error!("{e:?}")).unwrap();
wr.watch(&PathBuf::from("."), RecursiveMode::Recursive).unwrap();
loop {
    tokio::select! {
        _ = your_job => {},
        _ = match_event(rx, &exclude) => {
            // do your logic on fs update
        },
    }
}
```
