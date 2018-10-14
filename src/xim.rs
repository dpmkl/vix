use crate::command_prompt::{Command, CommandPrompt};
use crate::editor::Editor;
use crate::tty::{Tty, TtyEvent};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{future, Async, Future, Poll, Sink, Stream};
use std::io::{self, Write};
use termion::event::{Event, Key};
use tokio;
use xrl::{
    AvailablePlugins, Client, ConfigChanged, Frontend, FrontendBuilder, PluginStarted,
    PluginStoped, ScrollTo, ServerResult, Style, ThemeChanged, Update, UpdateCmds,
};

#[derive(Debug)]
pub enum CoreEvent {
    Update(Update),
    ScrollTo(ScrollTo),
    SetStyle(Style),
}

#[derive(Debug)]
pub enum Mode {
    Xim,
    Command,
    Search,
    Insert,
}

pub struct Xim {
    editor: Editor,
    mode: Mode,
    prompt: Option<CommandPrompt>,
    tty: Tty,
    tty_size: (u16, u16),
    shutdown: bool,
}

impl Xim {
    pub fn new(
        mut client: Client,
        events: UnboundedReceiver<CoreEvent>,
    ) -> Result<Self, io::Error> {
        let mut dir = dirs::config_dir().unwrap();
        dir.push("xim");
        tokio::run(client.client_started(dir.to_str(), None).map_err(|_| ()));

        Ok(Xim {
            editor: Editor::new(client, events),
            mode: Mode::Xim,
            prompt: None,
            tty: Tty::new()?,
            tty_size: (0, 0),
            shutdown: false,
        })
    }

    fn handle_resize(&mut self, size: (u16, u16)) {
        self.tty_size = size;
        self.editor.handle_resize(size);
    }

    fn exit(&mut self) {
        self.shutdown = true;
    }

    pub fn handle_cmd(&mut self, cmd: Command) {
        self.mode = Mode::Xim;
        match cmd {
            Command::Quit => {
                info!("exiting ...");
                self.exit();
            }
            Command::Cancel => {
                self.prompt = None;
            }
            Command::Save(view, exit) => {
                self.editor.save(view);
                if exit {
                    self.exit();
                }
            }
            Command::Open(file) => {
                self.editor.open(file);
            }
            Command::SetTheme(theme) => {
                self.editor.set_theme(&theme);
            }
            Command::Search(_search) => {}
            Command::GotoLine(line) => {
                let line = match line {
                    0 => 0,
                    _ => line -1,
                };
                self.editor.goto_line(line);
            }
            // Command::NextBuffer => self.editor.next_buffer(),
            // Command::PrevBuffer => self.editor.prev_buffer()
        }
    }

    fn handle_input(&mut self, event: Event) {
        debug!("event: {:?}@{:?}", event, self.mode);
        match event {
            Event::Key(Key::Ctrl('c')) => self.exit(),
            Event::Key(Key::Esc) => {
                info!("entering xim mode");
                self.mode = Mode::Xim;
            }
            Event::Key(key) => match self.mode {
                Mode::Xim => match key {
                    Key::Delete
                    | Key::Left
                    | Key::Right
                    | Key::Up
                    | Key::Down
                    | Key::PageUp
                    | Key::PageDown => self.editor.handle_input(event),
                    Key::Char('i') => {
                        info!("entering insert mode");
                        self.mode = Mode::Insert;
                    }
                    Key::Char(':') => {
                        info!("entering command mode");
                        self.mode = Mode::Command;
                        self.prompt = Some(CommandPrompt::execute());
                    }
                    Key::Char('/') => {
                        info!("entering search mode");
                        self.mode = Mode::Search;
                        self.prompt = Some(CommandPrompt::search());
                    }
                    _ => {}
                },
                Mode::Command => {
                    self.handle_command_prompt(event.clone());
                }
                Mode::Insert => self.editor.handle_input(event),
                Mode::Search => {
                    self.handle_command_prompt(event.clone());
                }
            },
            event => {
                if self.prompt.is_none() {
                    self.editor.handle_input(event);
                } else {
                    self.handle_command_prompt(event.clone());
                }
            }
        }
    }

    fn handle_command_prompt(&mut self, event: Event) {
        if self.prompt.is_some() {
            let mut prompt = self.prompt.take().unwrap();
            match prompt.handle_input(&event) {
                Ok(None) => {
                    self.prompt = Some(prompt);
                }
                Ok(Some(cmd)) => {
                    self.handle_cmd(cmd);
                    self.mode = Mode::Xim;
                }
                Err(err) => {
                    self.mode = Mode::Xim;
                    error!("failed to parse cmd: {:?}", err);
                }
            }
        }
    }

    fn process_terminal_events(&mut self) {
        let mut new_size: Option<(u16, u16)> = None;
        loop {
            match self.tty.poll() {
                Ok(Async::Ready(Some(event))) => match event {
                    TtyEvent::Resize(size) => {
                        new_size = Some(size);
                    }
                    TtyEvent::Input(input) => self.handle_input(input),
                },
                Ok(Async::Ready(None)) => {
                    error!("Terminal stream shutdown, exiting ...");
                    self.shutdown = true;
                }
                Ok(Async::NotReady) => break,
                Err(_) => {
                    error!("");
                    self.shutdown = true;
                }
            }
        }
        if let Some(size) = new_size {
            if !self.shutdown {
                self.handle_resize(size);
            }
        }
    }

    fn render(&mut self) -> Result<(), io::Error> {
        if let Some(ref mut prompt) = self.prompt {
            prompt.render(self.tty.stdout(), self.tty_size.1)?;
        } else {
            self.editor.render(self.tty.stdout());
        }
        if let Err(err) = self.tty.stdout().flush() {
            error!("Failed to flush stdout: {}", err);
        }
        Ok(())
    }
}

impl Future for Xim {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.editor.process_open_requests();
        self.editor.process_delayed_events();
        self.process_terminal_events();
        self.editor.process_core_events();
        if let Err(err) = self.render() {
            error!("Error rendering: {}", err);
        }
        if self.shutdown {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

pub struct XimService(UnboundedSender<CoreEvent>);

impl XimService {
    fn send_core_event(&mut self, event: CoreEvent) -> ServerResult<()> {
        if let Err(err) = self.0.start_send(event) {
            let e = format!("Error starting send core event {}", err);
            error!("{}", e);
            return Box::new(future::err(e.into()));
        }
        match self.0.poll_complete() {
            Ok(_) => Box::new(future::ok(())),
            Err(err) => {
                let e = format!("Error completing send core event {}", err);
                error!("{}", e);
                return Box::new(future::err(e.into()));
            }
        }
    }
}

impl Frontend for XimService {
    fn update(&mut self, update: Update) -> ServerResult<()> {
        self.send_core_event(CoreEvent::Update(update))
    }

    fn scroll_to(&mut self, scroll_to: ScrollTo) -> ServerResult<()> {
        self.send_core_event(CoreEvent::ScrollTo(scroll_to))
    }

    fn def_style(&mut self, style: Style) -> ServerResult<()> {
        self.send_core_event(CoreEvent::SetStyle(style))
    }

    fn available_plugins(&mut self, _plugins: AvailablePlugins) -> ServerResult<()> {
        warn!("AvailablePlugins not implemented");
        Box::new(future::ok(()))
    }

    fn update_cmds(&mut self, _update_cmds: UpdateCmds) -> ServerResult<()> {
        warn!("UpateCmds not implemented");
        Box::new(future::ok(()))
    }

    fn plugin_started(&mut self, _plugin: PluginStarted) -> ServerResult<()> {
        warn!("PluginStarted not implemented");
        Box::new(future::ok(()))
    }

    fn plugin_stoped(&mut self, _plugin: PluginStoped) -> ServerResult<()> {
        warn!("PluginStoped not implemented");
        Box::new(future::ok(()))
    }

    fn config_changed(&mut self, _config: ConfigChanged) -> ServerResult<()> {
        warn!("ConfigChanged not implemented");
        Box::new(future::ok(()))
    }

    fn theme_changed(&mut self, _theme: ThemeChanged) -> ServerResult<()> {
        warn!("ThemeChanged not implemented");
        Box::new(future::ok(()))
    }
}

pub struct XimServiceBuilder(UnboundedSender<CoreEvent>);

impl XimServiceBuilder {
    pub fn new() -> (Self, UnboundedReceiver<CoreEvent>) {
        let (tx, rx) = unbounded();
        (XimServiceBuilder(tx), rx)
    }
}

impl FrontendBuilder<XimService> for XimServiceBuilder {
    fn build(self, _client: Client) -> XimService {
        XimService(self.0)
    }
}
