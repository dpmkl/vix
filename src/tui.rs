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

pub struct Tui {
    editor: Editor,
    prompt: Option<CommandPrompt>,
    tty: Tty,
    tty_size: (u16, u16),
    shutdown: bool,
}
