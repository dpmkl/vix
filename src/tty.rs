use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{Async, Poll, Sink, Stream};
use std::io::{self, Stdout};
use std::thread::{sleep, spawn};
use std::time::Duration;
use termion::event::Event;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::terminal_size;

pub type RenderTarget = MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>;

pub struct Tty {
    size: UnboundedReceiver<(u16, u16)>,
    stdin: UnboundedReceiver<Event>,
    stdout: RenderTarget,
}

impl Tty {
    pub fn new() -> Result<Self, io::Error> {
        let (stdin_tx, stdin_rx) = unbounded();
        let (size_tx, size_rx) = unbounded();
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(AlternateScreen::from(stdout));

        let tty = Tty {
            size: size_rx,
            stdin: stdin_rx,
            stdout,
        };

        Tty::start_stdin_listening(stdin_tx);
        Tty::start_size_listening(size_tx);

        Ok(tty)
    }

    fn start_stdin_listening(tx: UnboundedSender<Event>) {
        let mut tx = tx;
        spawn(move || {
            for event in io::stdin().events() {
                match event {
                    Ok(event) => {
                        let _ = tx.start_send(event).unwrap();
                        let _ = tx.poll_complete().unwrap();
                    }
                    Err(err) => error!("failed to read tty event:  {}", err),
                }
            }
        });
    }
    fn start_size_listening(tx: UnboundedSender<(u16, u16)>) {
        let mut tx = tx;
        spawn(move || {
            let mut current_size = (0, 0);
            info!("waiting for resize events");
            loop {
                match terminal_size() {
                    Ok(new_size) => {
                        if new_size != current_size {
                            info!(
                                "terminal resized (from {:?} to {:?})",
                                current_size, new_size
                            );
                            current_size = new_size;
                            let _ = tx.start_send(current_size).unwrap();
                            let _ = tx.poll_complete().unwrap();
                        }
                    }
                    Err(e) => {
                        error!("failed to get terminal size: {}", e);
                    }
                }
                sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn stdout(&mut self) -> &mut RenderTarget {
        &mut self.stdout
    }
}

pub enum TtyEvent {
    Resize((u16, u16)),
    Input(Event),
}

impl Stream for Tty {
    type Item = TtyEvent;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.size.poll() {
            Ok(Async::Ready(Some(size))) => {
                return Ok(Async::Ready(Some(TtyEvent::Resize(size))));
            }
            Ok(Async::Ready(None)) => {
                return Ok(Async::Ready(None));
            }
            Ok(Async::NotReady) => {}
            Err(()) => return Err(()),
        }
        match self.stdin.poll() {
            Ok(Async::Ready(Some(event))) => return Ok(Async::Ready(Some(TtyEvent::Input(event)))),
            Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
            Ok(Async::NotReady) => {}
            Err(()) => return Err(()),
        }
        Ok(Async::NotReady)
    }
}
