mod client;
mod command_prompt;
mod editor;
mod tty;
mod tui;
mod view;
mod window;

use slog::Drain;
use slog_scope::GlobalLoggerGuard;
use std::fs::OpenOptions;

#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;

fn setup_log(file: Option<String>) -> GlobalLoggerGuard {
    let drain = match file {
        Some(file) => {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file)
                .unwrap();

            let decorator = slog_term::PlainDecorator::new(file);
            let drain = slog_term::FullFormat::new(decorator).build().fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            drain
        }
        None => {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = slog_term::CompactFormat::new(decorator).build().fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            drain
        }
    };
    let log = slog::Logger::root(drain, o!());
    slog_scope::set_global_logger(log)
}

fn main() {
    println!("Hello, world!");
}
