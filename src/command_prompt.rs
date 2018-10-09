use std::io::{Error, Write};
use std::str::FromStr;
use termion::clear::CurrentLine;
use termion::cursor::Goto;
use termion::event::{Event, Key};
use xrl::ViewId;

#[derive(Debug)]
pub enum Command {
    Cancel,
    Quit,
    Write(Option<ViewId>),
    WriteQuit(Option<ViewId>),
    Open(Option<String>),
    SetSyntax(String),
}

#[derive(Debug)]
pub enum ParseCommandError {
    UnexpectedArgument,
    ExpectedArgument {
        cmd: String,
        expected: usize,
        found: usize,
    },
    TooManyArguments {
        cmd: String,
        expected: usize,
        found: usize,
    },
    UnknownCommand(String),
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Command, Self::Err> {
        match &s[..] {
            "w" | "write" => Ok(Command::Write(None)),
            "wq" | "writequit" => Ok(Command::WriteQuit(None)),
            command => {
                let mut parts: Vec<&str> = command.split(' ').collect();
                let cmd = parts.remove(0);
                match cmd {
                    _ => Err(ParseCommandError::UnknownCommand(command.into())),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandPrompt {
    index: usize,
    chars: String,
}

impl CommandPrompt {
    pub fn handle_input(&mut self, input: &Event) -> Result<Option<Command>, ParseCommandError> {
        match input {
            Event::Key(Key::Char('\n')) => self.finalize(),
            Event::Key(Key::Backspace) => Ok(self.back()),
            Event::Key(Key::Delete) => Ok(self.delete()),
            Event::Key(Key::Left) => Ok(self.left()),
            Event::Key(Key::Right) => Ok(self.right()),
            Event::Key(Key::Char(chr)) => Ok(self.new_key(*chr)),
            _ => Ok(None),
        }
    }

    fn left(&mut self) -> Option<Command> {
        if self.index > 0 {
            self.index -= 1;
        }
        None
    }

    fn right(&mut self) -> Option<Command> {
        if self.index < self.chars.len() {
            self.index += 1;
        }
        None
    }

    fn delete(&mut self) -> Option<Command> {
        if self.index < self.chars.len() {
            self.chars.remove(self.index);
        }
        None
    }

    fn back(&mut self) -> Option<Command> {
        if self.chars.is_empty() {
            Some(Command::Cancel)
        } else {
            self.index -= 1;
            self.chars.remove(self.index);
            None
        }
    }

    fn new_key(&mut self, chr: char) -> Option<Command> {
        self.chars.insert(self.index, chr);
        self.index += 1;
        None
    }

    fn finalize(&mut self) -> Result<Option<Command>, ParseCommandError> {
        Ok(Some(FromStr::from_str(&self.chars)?))
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        if let Err(err) = write!(
            w,
            "{}{}:{}{}",
            Goto(1, row),
            CurrentLine,
            self.chars,
            Goto(self.index as u16 + 2, row)
        ) {
            println!("{}", err);
        }
        Ok(())
    }
}
