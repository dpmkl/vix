use crate::view::Cursor;

#[derive(Clone, Debug)]
pub struct Window {
    start: u64,
    size: u16,
}

impl Window {
    pub fn new() -> Self {
        Window { start: 0, size: 0 }
    }

    pub fn set_cursor(&mut self, cursor: &Cursor) {
        debug!("setting cursor to {:?}", cursor);
        if cursor.line < self.start() {
            self.start = cursor.line;
        } else if cursor.line >= self.end() {
            self.start = 1 + cursor.line - u64::from(self.size);
        }
        debug!("new window: {:?}", self);
    }

    pub fn resize(&mut self, height: u16) {
        self.size = height;
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        u64::from(self.size) + self.start
    }

    pub fn update(&mut self, cursor: u64, nb_line: u64) {
        debug!(
            "resizing window: height: {}; cursor: {}; nb_line: {}",
            self.size, cursor, nb_line
        );

        let mut new_start = if u64::from(self.size) > self.start() + self.end() {
            0
        } else {
            (self.start + self.end() - u64::from(self.size)) / 2
        };

        if new_start + u64::from(self.size) > nb_line {
            if nb_line < u64::from(self.size) {
                new_start = 0;
            } else {
                new_start = nb_line - u64::from(self.size);
            }
        }
        if cursor < new_start {
            new_start = cursor;
        } else if cursor > new_start + u64::from(self.size) {
            new_start = cursor - u64::from(self.size);
        }

        self.start = new_start;
        debug!("resized window: {:?}", self);
    }
}
