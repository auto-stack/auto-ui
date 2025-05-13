use crate::dyna::DynaStory;
use auto_ui::*;
use gpui::*;
use std::time::Duration;

use notify::event::ModifyKind;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Result, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

fn watch_at() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // No specific tickrate, max debounce time 1 seconds
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

    debouncer.watcher().watch(
        Path::new("crates/auto-ui/examples/dyna_hello.at"),
        RecursiveMode::NonRecursive,
    )?;

    // Use recommended_watcher() to automatically select the best implementation
    // for your platform. The `EventHandler` passed to this constructor can be a
    // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
    // another type the trait is implemented for.
    // let mut watcher = notify::recommended_watcher(tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    // watcher.watch(
    // Path::new("crates/auto-ui/examples/dyna_hello.at"),
    // RecursiveMode::Recursive,
    // )?;
    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(events) => {
                for ev in events {
                    println!("Event: {:?}", ev);
                    if let DebouncedEventKind::Any = ev.kind {
                        println!("file changed!");
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

/// Async, futures channel based event watching
fn main() {
    println!("starting UI...");

    thread::spawn(|| {
        watch_at().expect("haha");
    });

    let app = Application::new().with_assets(Assets);
    app.run(move |cx| {
        init(cx);
        cx.activate(true);
        create_new_window_sized("Hello Example", StoryView::view::<DynaStory>, cx, 800, 600);
    });
}
