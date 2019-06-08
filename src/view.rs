use crate::client::Client;
use crate::style::{reset_style, set_style};
use crate::window::Window;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
//use termion;
use termion::clear::CurrentLine;
use termion::color;
use termion::cursor::Goto;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::style::{Bold, Reset};
use xrl::{ClientResult, Line, LineCache, ModifySelection, Style, Update};

#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub line: u64,
    pub column: u64,
}

pub struct View {
    cache: LineCache,
    cursor: Cursor,
    window: Window,
    file: Option<String>,
    client: Client,
    gutter_size: u16,
    tab_width: u16,
}

impl View {
    pub fn new(client: Client, file: Option<String>) -> View {
        View {
            client,
            cache: LineCache::default(),
            cursor: Default::default(),
            window: Window::new(),
            file,
            gutter_size: 0,
            tab_width: 4,
        }
    }

    pub fn update_cache(&mut self, update: Update) {
        debug!("updating cache");
        self.cache.update(update)
    }

    pub fn set_cursor(&mut self, line: u64, column: u64) {
        self.cursor = Cursor { line, column };
        self.window.set_cursor(&self.cursor);
    }

    pub fn render<W: Write>(&mut self, w: &mut W, styles: &HashMap<u64, Style>, state: &str) {
        self.update_window();
        self.render_lines(w, styles);
        self.render_status(w, state);
        self.render_cursor(w);
    }

    pub fn resize(&mut self, height: u16) {
        self.window.resize(height);
        self.update_window();
        self.client.scroll(
            self.cache.before() + self.window.start(),
            self.cache.after() + self.window.end(),
        );
    }

    pub fn collapse_selections(&mut self) {
        self.client.collapse_selections();
    }

    pub fn select_line(&mut self) {
        self.client.select_line();
    }

    pub fn select_line_end(&mut self) {
        self.client.select_line_end();
    }

    pub fn delete_line(&mut self) {
        self.client.delete_line();
    }

    pub fn goto_line(&mut self, line: u64) {
        self.client.goto_line(line)
    }

    pub fn copy(&mut self) -> ClientResult<Value> {
        self.client.copy()
    }

    pub fn paste(&mut self, buffer: &str) {
        self.client.paste(buffer);
    }

    pub fn cut(&mut self) -> ClientResult<Value> {
        self.client.cut()
    }

    pub fn undo(&mut self) {
        self.client.undo();
    }

    pub fn redo(&mut self) {
        self.client.redo();
    }

    pub fn find(
        &mut self,
        search_term: &str,
        case_sensitive: bool,
        regex: bool,
        whole_words: bool,
    ) {
        self.client
            .find(search_term, case_sensitive, regex, whole_words);
    }

    pub fn find_next(
        &mut self,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) {
        self.client
            .find_next(wrap_around, allow_same, modify_selection);
    }

    pub fn find_prev(
        &mut self,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) {
        self.client
            .find_prev(wrap_around, allow_same, modify_selection);
    }

    pub fn find_all(&mut self) {
        self.client.find_all();
    }

    pub fn highlight_find(&mut self, visible: bool) {
        self.client.highlight_find(visible);
    }

    pub fn save(&mut self) -> ClientResult<()> {
        self.client.save(self.file.as_ref().unwrap())
    }

    fn update_window(&mut self) {
        if self.cursor.line < self.cache.before() {
            error!(
                "cursor is on line {} but there are {} invalid lines in cache.",
                self.cursor.line,
                self.cache.before()
            );
            return;
        }
        let cursor_line = self.cursor.line - self.cache.before();
        let nb_lines = self.cache.lines().len() as u64;
        self.gutter_size = 1
            + (self.cache.before() + nb_lines + self.cache.after())
                .to_string()
                .len() as u16;
        self.window.update(cursor_line, nb_lines);
    }

    fn get_click_location(&self, x: u64, y: u64) -> (u64, u64) {
        let lineno = x + self.cache.before() + self.window.start();
        if let Some(line) = self.cache.lines().get(x as usize) {
            if y == 0 {
                return (lineno, 0);
            }
            let mut text_len: u16 = 0;
            for (idx, c) in line.text.chars().enumerate() {
                text_len = self.translate_char_width(text_len, c);
                if u64::from(text_len) >= y {
                    return (lineno as u64, idx as u64 + 1);
                }
            }
            return (lineno, line.text.len() as u64 + 1);
        } else {
            warn!("no line at index {} found in cache", x);
            return (x, y);
        }
    }

    fn click(&mut self, x: u64, y: u64) {
        let (line, column) = self.get_click_location(x, y);
        self.client.click(line, column);
    }

    fn drag(&mut self, x: u64, y: u64) {
        let (line, column) = self.get_click_location(x, y);
        self.client.drag(line, column);
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(key) => match key {
                Key::Char(c) => self.client.insert(c),
                // FIXME: avoid complexity
                // Key::Ctrl(c) => match c {
                //     'w' => self.client.save(self.file.as_ref().unwrap()),
                //     'h' => self.client.backspace(),
                //     _ => error!("Unhandled input ctrl+{}", c),
                // },
                Key::Backspace => self.client.backspace(),
                Key::Delete => self.client.delete(),
                Key::Left => self.client.left(),
                Key::Right => self.client.right(),
                Key::Up => self.client.up(),
                Key::Down => self.client.down(),
                Key::Home => self.client.home(),
                Key::End => self.client.end(),
                Key::PageUp => self.client.page_up(),
                Key::PageDown => self.client.page_down(),
                k => error!("unhandled key {:?}", k),
            },
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Press(press_event, y, x) => match press_event {
                    MouseButton::Left => self.click(u64::from(x) - 1, u64::from(y) - 1),
                    MouseButton::WheelUp => self.client.up(),
                    MouseButton::WheelDown => self.client.down(),
                    button => error!("unhandled button {:?}", button),
                },
                MouseEvent::Release(..) => {}
                MouseEvent::Hold(y, x) => self.drag(u64::from(x) - 1, u64::from(y) - 1),
            },
            ev => error!("unhandled event {:?}", ev),
        }
    }

    pub fn render_error<W: Write>(&mut self, w: &mut W, msg: &str) {
        let win_size = self.window.size() + 1;
        write!(
            w,
            "{}{}{}error{}{} : {}",
            Goto(1, win_size),
            CurrentLine,
            Bold,
            color::Fg(color::Red),
            Reset,
            msg
        )
        .unwrap();
    }

    fn render_lines<W: Write>(&self, w: &mut W, styles: &HashMap<u64, Style>) {
        debug!("rendering lines");
        trace!("current cache\n{:?}", self.cache);

        let lines = self
            .cache
            .lines()
            .iter()
            .skip(self.window.start() as usize)
            .take(self.window.size() as usize);

        let mut line_strings = String::new();
        let mut line_no = self.cache.before() + self.window.start();
        for (line_index, line) in lines.enumerate() {
            line_strings.push_str(&self.render_line_str(line, Some(line_no), line_index, styles));
            line_no += 1;
        }

        let line_count = self.cache.lines().len() as u16;
        let win_size = self.window.size();
        if win_size > line_count {
            for num in line_count..win_size {
                line_strings.push_str(&self.render_line_str(
                    &Line::default(),
                    None,
                    num as usize,
                    styles,
                ));
            }
        }
        w.write_all(line_strings.as_bytes()).unwrap();
    }

    fn render_status<W: Write>(&mut self, w: &mut W, state: &str) {
        let win_size = self.window.size() + 1;
        let file = match self.file.as_ref().map(|s| s) {
            None => "<nofile>".to_owned(),
            Some(file) => file.to_owned(),
        };
        let cur = self.window.get_cursor();
        write!(
            w,
            "{}{}{}{}{}{} : '{}' {} / {}",
            Goto(1, win_size),
            CurrentLine,
            Bold,
            color::Fg(color::Green),
            state,
            Reset,
            file,
            cur.line + 1,
            cur.column + 1
        )
        .unwrap();
    }

    fn render_line_str(
        &self,
        line: &Line,
        lineno: Option<u64>,
        line_index: usize,
        styles: &HashMap<u64, Style>,
    ) -> String {
        let text = self.escape_control_and_add_styles(styles, line);
        if let Some(line_no) = lineno {
            format!(
                "{}{}{}{}{}",
                Goto(1, line_index as u16 + 1),
                CurrentLine,
                (line_no + 1).to_string(),
                Goto(self.gutter_size + 1, line_index as u16 + 1),
                &text
            )
        } else {
            format!(
                "{}{}~{}",
                Goto(self.gutter_size + 1, line_index as u16 + 1),
                CurrentLine,
                &text
            )
        }
    }

    fn escape_control_and_add_styles(&self, styles: &HashMap<u64, Style>, line: &Line) -> String {
        let mut position: u16 = 0;
        let mut text = String::with_capacity(line.text.capacity());
        for c in line.text.chars() {
            match c {
                '\x00'...'\x08' | '\x0a'...'\x1f' | '\x7f' => {
                    // Render in caret notation, i.e. '\x02' is rendered as '^B'
                    text.push('^');
                    text.push((c as u8 ^ 0x40u8) as char);
                    position += 2;
                }
                '\t' => {
                    let tab_width = self.tab_width_at_position(position);
                    text.push_str(&" ".repeat(tab_width as usize));
                    position += tab_width;
                }
                _ => {
                    text.push(c);
                    position += 1;
                }
            }
        }
        if line.styles.is_empty() {
            return text;
        }
        let mut style_sequences = self.get_style_sequences(styles, line);
        for style in style_sequences.drain(..) {
            trace!("inserting style: {:?}", style);
            if style.0 >= text.len() {
                text.push_str(&style.1);
            } else {
                text.insert_str(style.0, &style.1);
            }
        }
        trace!("styled line: {:?}", text);
        text
    }

    fn tab_width_at_position(&self, position: u16) -> u16 {
        self.tab_width - (position % self.tab_width)
    }

    fn get_style_sequences(
        &self,
        styles: &HashMap<u64, Style>,
        line: &Line,
    ) -> Vec<(usize, String)> {
        let mut style_sequences: Vec<(usize, String)> = Vec::new();
        let mut prev_style_end: usize = 0;
        for style_def in &line.styles {
            let start_idx = if style_def.offset >= 0 {
                (prev_style_end + style_def.offset as usize)
            } else {
                (prev_style_end - ((-style_def.offset) as usize))
            };
            let end_idx = start_idx + style_def.length as usize;
            prev_style_end = end_idx;

            if let Some(style) = styles.get(&style_def.style_id) {
                let start_sequence = match set_style(style) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("could not get CSI sequence to set style {:?}: {}", style, e);
                        continue;
                    }
                };
                let end_sequence = match reset_style(style) {
                    Ok(s) => s,
                    Err(e) => {
                        error!(
                            "could not get CSI sequence to reset style {:?}: {}",
                            style, e
                        );
                        continue;
                    }
                };
                style_sequences.push((start_idx, start_sequence));
                style_sequences.push((end_idx, end_sequence));
            } else {
                error!(
                    "no style ID {} found not applying style.",
                    style_def.style_id
                );
            };
        }

        style_sequences.sort_by(|a, b| a.0.cmp(&b.0));
        style_sequences.reverse();
        trace!("{:?}", style_sequences);
        style_sequences
    }

    fn render_cursor<W: Write>(&self, w: &mut W) {
        debug!("rendering cursor");
        if self.cache.is_empty() {
            debug!("cache is empty, rendering cursor at the top left corner");
            if let Err(e) = write!(w, "{}", Goto(1, 1)) {
                error!("failed to render cursor: {}", e);
            }
            return;
        }

        if self.cursor.line < self.cache.before() {
            error!(
                "The cursor is on line {} which is marked invalid in the cache",
                self.cursor.line
            );
            return;
        }
        let line_idx = self.cursor.line - self.cache.before();
        let line = match self.cache.lines().get(line_idx as usize) {
            Some(line) => line,
            None => {
                error!("no valid line at cursor index {}", self.cursor.line);
                return;
            }
        };

        if line_idx < self.window.start() {
            error!(
                "the line that has the cursor (nb={}, cache_idx={}) not within the displayed window ({:?})",
                self.cursor.line,
                line_idx,
                self.window
            );
            return;
        }
        let line_pos = line_idx - self.window.start();

        let column = line
            .text
            .chars()
            .take(self.cursor.column as usize)
            .fold(0, |acc, c| acc + self.translate_char_width(acc, c));

        let cursor_pos = Goto(self.gutter_size + column as u16 + 1, line_pos as u16 + 1);
        if let Err(e) = write!(w, "{}", cursor_pos) {
            error!("failed to render cursor: {}", e);
        }
        debug!("cursor rendered at ({}, {})", line_pos, column);
    }

    fn translate_char_width(&self, position: u16, c: char) -> u16 {
        match c {
            '\x00'...'\x08' | '\x0a'...'\x1f' | '\x7f' => 2,
            '\t' => self.tab_width_at_position(position),
            _ => 1,
        }
    }
}
