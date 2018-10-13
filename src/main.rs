mod client;
mod command_prompt;
mod editor;
mod style;
mod tty;
mod view;
mod window;
mod xim;

use futures::{Future, Stream};
use slog::{Drain, Level, LevelFilter};
use slog_scope::GlobalLoggerGuard;
use std::env;
use std::fs::OpenOptions;
use xrl;

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
            let drain = slog_term::CompactFormat::new(decorator).build().fuse();
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
    let log = slog::Logger::root(LevelFilter::new(drain, Level::Info).fuse(), o!());
    slog_scope::set_global_logger(log)
}

fn main() {
    let _guard = setup_log(Some("xim.log".to_owned()));
    let mut args = Vec::new();
    args.extend(std::env::args().skip(1));
    if args.len() == 0 {
        args.push("Cargo.toml".to_owned());
    }

    info!("Starting xi-core");
    let (xim_builder, core_events_rx) = xim::XimServiceBuilder::new();
    let (client, core_stderr) = xrl::spawn("xi-core", xim_builder);

    let err_log = core_stderr
        .for_each(|msg| {
            error!("xi-core| {}", msg);
            Ok(())
        })
        .map_err(|_| ());
    ::std::thread::spawn(move || {
        tokio::run(err_log);
    });

    info!("Starting xim");
    let mut xim = match xim::Xim::new(client, core_events_rx) {
        Ok(xim) => xim,
        Err(err) => {
            error!("Error starting xim: {}", err);
            ::std::process::exit(1);
        }
    };

    for file in args {
        xim.handle_cmd(command_prompt::Command::Open(Some(file)));
    }

    tokio::run(xim.map_err(|err| {
        error!("{}", err);
        ()
    }));
}
