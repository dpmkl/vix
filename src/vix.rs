use crate::command_prompt::{Command, CommandPrompt};
use crate::editor::Editor;
use crate::tty::{Tty, TtyEvent};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{future, Async, Future, Poll, Sink, Stream};
use std::io::{self, Write};
use termion::event::{Event, Key};
use tokio;
use xrl::{
    AvailablePlugins, Client, ConfigChanged, Frontend, FrontendBuilder, ModifySelection,
    PluginStarted, PluginStoped, ScrollTo, ServerResult, Style, ThemeChanged, Update, UpdateCmds,
};

#[derive(Debug)]
pub enum CoreEvent {
    Update(Update),
    ScrollTo(ScrollTo),
    SetStyle(Style),
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Vix,
    Error(String),
    Command,
    Search,
    Insert,
    Visual(bool),
}

pub struct Vix {
    editor: Editor,
    mode: Mode,
    prompt: Option<CommandPrompt>,
    tty: Tty,
    tty_size: (u16, u16),
    shutdown: bool,
    last_event: Event,
}

impl Vix {
    pub fn new(
        mut client: Client,
        events: UnboundedReceiver<CoreEvent>,
    ) -> Result<Self, io::Error> {
        let mut dir = dirs::config_dir().unwrap();
        dir.push("vix");
        tokio::run(client.client_started(dir.to_str(), None).map_err(|_| ()));

        Ok(Vix {
            editor: Editor::new(client, events),
            mode: Mode::Vix,
            prompt: None,
            tty: Tty::new()?,
            tty_size: (0, 0),
            shutdown: false,
            last_event: Event::Unsupported(Vec::new()),
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
        self.mode = Mode::Vix;
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
            Command::Search(search) => {
                self.editor.find(&search, true, false, false);
                self.editor.find_all();
                self.editor.highlight_find(true);
            }
            Command::GotoLine(line) => {
                let line = match line {
                    0 => 0,
                    _ => line - 1,
                };
                self.editor.goto_line(line);
            }
        }
    }

    fn handle_input(&mut self, event: Event) {
        debug!("event: {:?}@{:?}", event, self.mode);
        match event.clone() {
            Event::Key(Key::Ctrl('c')) => self.exit(),
            Event::Key(Key::Esc) => {
                info!("entering vix mode");
                self.mode = Mode::Vix;
            }
            Event::Key(key) => match &self.mode {
                Mode::Error(_msg) => {
                    self.mode = Mode::Vix;
                }
                Mode::Visual(line_mode) => match key {
                    Key::Char('i') => self.mode = Mode::Insert,
                    Key::Char('p') => {
                        self.editor.paste();
                        self.mode = Mode::Vix
                    }
                    Key::Char('y') => {
                        self.editor.copy();
                        self.mode = Mode::Vix
                    }
                    Key::Char('d') => {
                        self.editor.cut();
                        self.mode = Mode::Vix
                    }
                    Key::Left => {
                        if !*line_mode {
                            self.editor.select_left();
                        }
                    }
                    Key::Right => {
                        if !*line_mode {
                            self.editor.select_right();
                        }
                    }
                    Key::Down => {
                        self.editor.select_down();
                        if *line_mode {
                            self.editor.select_line_end();
                        }
                    }
                    Key::Up => {
                        self.editor.select_up();
                        if *line_mode {
                            self.editor.select_line_end();
                        }
                    }
                    Key::PageUp => {
                        self.editor.select_page_up();
                        if *line_mode {
                            self.editor.select_line_end();
                        }
                    }
                    Key::PageDown => {
                        self.editor.select_page_down();
                        if *line_mode {
                            self.editor.select_line_end();
                        }
                    }
                    Key::Home => self.editor.select_home(),
                    Key::End => self.editor.select_end(),
                    Key::Char('j') => {
                        self.editor.select_down();
                        if *line_mode {
                            self.editor.select_line_end();
                        }
                    }
                    Key::Char('k') => {
                        self.editor.select_up();
                        if *line_mode {
                            self.editor.select_line_end();
                        }
                    }
                    Key::Char('l') => self.editor.select_right(),
                    _ => {}
                },
                Mode::Vix => match key {
                    Key::Delete
                    | Key::Left
                    | Key::Right
                    | Key::Up
                    | Key::Down
                    | Key::Home
                    | Key::End
                    | Key::PageUp
                    | Key::PageDown => self.editor.handle_input(event.clone()),
                    Key::Char('j') => self.editor.down(),
                    Key::Char('k') => self.editor.up(),
                    Key::Char('l') => self.editor.right(),
                    Key::Char(':') => {
                        info!("entering command mode");
                        self.mode = Mode::Command;
                        self.prompt = Some(CommandPrompt::execute());
                    }
                    Key::Char('i') => {
                        info!("entering insert mode");
                        self.mode = Mode::Insert;
                    }
                    Key::Char('/') => {
                        info!("entering search mode");
                        self.mode = Mode::Search;
                        self.prompt = Some(CommandPrompt::search());
                    }
                    Key::Char('v') => {
                        info!("entering visual mode");
                        self.mode = Mode::Visual(false);
                    }
                    Key::Char('V') => {
                        info!("entering visual line mode, FIXME");
                        self.editor.select_line();
                        self.mode = Mode::Visual(true);
                    }
                    Key::Char('p') => {
                        self.editor.paste();
                    }
                    Key::Char('y') => {
                        self.editor.copy();
                    }
                    Key::Char('u') => {
                        self.editor.undo();
                    }
                    Key::Char('r') => {
                        self.editor.redo();
                    }
                    Key::Char('d') => {
                        if self.last_event == Event::Key(Key::Char('d')) {
                            self.editor.delete_line();
                        }
                    }
                    Key::Char('n') => {
                        self.editor.find_next(true, false, ModifySelection::None);
                        self.editor.highlight_find(true);
                    }
                    Key::Char('N') => {
                        self.editor.find_prev(true, false, ModifySelection::None);
                        self.editor.highlight_find(true);
                    }
                    _ => {}
                },
                Mode::Command => {
                    self.handle_command_prompt(&event.clone());
                }
                Mode::Insert => self.editor.handle_input(event.clone()),
                Mode::Search => {
                    self.handle_command_prompt(&event.clone());
                }
            },
            event => {
                if self.prompt.is_none() {
                    self.editor.handle_input(event.clone());
                } else {
                    self.handle_command_prompt(&event.clone());
                }
            }
        }
        self.last_event = event;
    }

    fn handle_command_prompt(&mut self, event: &Event) {
        if self.prompt.is_some() {
            let mut prompt = self.prompt.take().unwrap();
            match prompt.handle_input(&event) {
                Ok(None) => {
                    self.prompt = Some(prompt);
                }
                Ok(Some(cmd)) => {
                    self.handle_cmd(cmd);
                    self.mode = Mode::Vix;
                }
                Err(err) => {
                    self.mode = Mode::Error(format!("Failed to parse cmd: '{:?}'!", err));
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
            let state = match self.mode {
                Mode::Vix => "vix",
                Mode::Insert => "insert",
                Mode::Visual(line_mode) => {
                    if line_mode {
                        "visual line"
                    } else {
                        "visual"
                    }
                }
                _ => "",
            };
            self.editor.render(self.tty.stdout(), state);
            if let Mode::Error(msg) = &self.mode {
                self.editor.render_error(self.tty.stdout(), msg);
            }
        }
        if let Err(err) = self.tty.stdout().flush() {
            error!("Failed to flush stdout: {}", err);
        }
        Ok(())
    }
}

impl Future for Vix {
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

pub struct VixService(UnboundedSender<CoreEvent>);

impl VixService {
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
                Box::new(future::err(e.into()))
            }
        }
    }
}

impl Frontend for VixService {
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
        warn!("AvailablePlugins not implemented: {:?}", _plugins);
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

    fn theme_changed(&mut self, theme: ThemeChanged) -> ServerResult<()> {
        warn!("ThemeChanged not implemented {:?}", theme);
        Box::new(future::ok(()))
    }
}

pub struct VixServiceBuilder(UnboundedSender<CoreEvent>);

impl VixServiceBuilder {
    pub fn new() -> (Self, UnboundedReceiver<CoreEvent>) {
        let (tx, rx) = unbounded();
        (VixServiceBuilder(tx), rx)
    }
}

impl FrontendBuilder<VixService> for VixServiceBuilder {
    fn build(self, _client: Client) -> VixService {
        VixService(self.0)
    }
}
