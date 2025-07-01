# fs-change-notifier

Simple library to watch file changes inside given directory.

Usage example:

```rust
use fs_change_notifier::{create_watcher, match_event, RecursiveMode};

let root = PathBuf::from(".");
let (mut wr, rx) = create_watcher(|e| log::error!("{e:?}")).unwrap();
wr.watch(&root, RecursiveMode::Recursive).unwrap();
loop {
    tokio::select! {
        _ = your_job => {},
        _ = match_event(&root, rx, &exclude) => {
            // do your logic on fs update
        },
    }
}
```
