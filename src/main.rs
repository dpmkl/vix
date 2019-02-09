mod client;
mod command_prompt;
mod editor;
mod style;
mod tty;
mod view;
mod vix;
mod window;

use futures::{Future, Stream};
use slog::{Drain, Level, LevelFilter};
use slog_scope::GlobalLoggerGuard;
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
    let _guard = setup_log(Some("vix.log".to_owned()));
    let mut args = Vec::new();
    args.extend(std::env::args().skip(1));

    if args.is_empty() {
        let help = "no file specified";
        crit!("{}", help);
        println!("{}", help);
        return;
    }

    info!("Starting xi-core");
    let (vix_builder, core_events_rx) = vix::VixServiceBuilder::new();
    let (client, core_stderr) = xrl::spawn("xi-core", vix_builder);

    let err_log = core_stderr
        .for_each(|msg| {
            error!("xi-core| {}", msg);
            Ok(())
        })
        .map_err(|_| ());
    ::std::thread::spawn(move || {
        tokio::run(err_log);
    });

    info!("Starting vix");
    let mut vix = match vix::Vix::new(client, core_events_rx) {
        Ok(vix) => vix,
        Err(err) => {
            error!("Error starting vix: {}", err);
            ::std::process::exit(1);
        }
    };
    vix.handle_cmd(command_prompt::Command::SetTheme(
        "base16-eighties.dark".to_owned(),
    ));
    for file in args {
        vix.handle_cmd(command_prompt::Command::Open(Some(file)));
    }

    tokio::run(vix.map_err(|err| {
        error!("{}", err);
        ()
    }));
}
