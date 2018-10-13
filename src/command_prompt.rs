use std::io::{Error, Write};
use std::str;
use std::str::FromStr;
use termion::clear::CurrentLine;
use termion::cursor::Goto;
use termion::event::{Event, Key};
use xrl::ViewId;

#[derive(Debug)]
pub enum Command {
    Search(String),
    Cancel,
    Quit,
    Save(Option<ViewId>, bool),
    GotoLine(u16),
    Open(Option<String>),
    //SetSyntax(String),
}

#[derive(Debug)]
pub enum ParseCommandError {
    /*
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
    },*/
    UnknownCommand(String),
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Command, Self::Err> {
        let num = s.to_owned();
        if let Ok(idx) = num.parse::<u16>() {
            Ok(Command::GotoLine(idx))
        } else {
            match &s[..] {
                // FIXME: Unsure how tow handle the ! operator here
                "w" | "write" => Ok(Command::Save(None, false)),
                "q" | "quit" => Ok(Command::Quit),
                // FIXME: Parent future (Xim) exits before save future is complete
                "wq" => Ok(Command::Save(None, true)),
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
}

#[derive(Debug)]
pub enum InputType {
    Command,
    Search,
}

#[derive(Debug)]
pub struct CommandPrompt {
    chars: String,
    index: usize,
    input_type: InputType,
    prefix: String,
}

impl CommandPrompt {
    pub fn search() -> Self {
        CommandPrompt {
            chars: "".to_string(),
            index: 0,
            input_type: InputType::Search,
            prefix: "(search):".to_string(),
        }
    }

    pub fn execute() -> Self {
        CommandPrompt {
            chars: "".to_string(),
            index: 0,
            input_type: InputType::Command,
            prefix: "(exec):".to_string(),
        }
    }

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
        match self.input_type {
            InputType::Command => Ok(Some(FromStr::from_str(&self.chars)?)),
            InputType::Search => Ok(Some(Command::Search(self.chars.clone()))),
        }
    }

    pub fn render<W: Write>(&mut self, w: &mut W, row: u16) -> Result<(), Error> {
        if let Err(err) = write!(
            w,
            "{}{}{}{}{}",
            Goto(1 as u16, row),
            CurrentLine,
            self.prefix,
            self.chars,
            Goto(self.prefix.len() as u16 + self.index as u16 + 1, row)
        ) {
            error!("{}", err);
        }
        Ok(())
    }
}
