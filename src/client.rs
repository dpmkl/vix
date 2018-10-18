use futures::Future;
use serde_json::Value;
use tokio;
use xrl;
use xrl::{ClientResult, ModifySelection};

pub struct Client {
    inner: xrl::Client,
    view_id: xrl::ViewId,
}

impl Client {
    pub fn new(client: xrl::Client, view_id: xrl::ViewId) -> Self {
        Client {
            inner: client,
            view_id,
        }
    }

    pub fn insert(&mut self, chr: char) {
        let f = self.inner.char(self.view_id, chr).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn scroll(&mut self, start: u64, end: u64) {
        let f = self.inner.scroll(self.view_id, start, end).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn select_line_end(&mut self) {
        let f = self.inner.line_end_sel(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn select_line(&mut self) {
        // FIXME: Find non blocking way by chaining
        self.inner.line_start(self.view_id).wait().unwrap();
        self.inner.line_end_sel(self.view_id).wait().unwrap();
        // let end_sel = self.inner.line_end_sel(self.view_id).map_err(|_| ());
        // let sel = self.inner.line_start(self.view_id).and_then(|_| {
        //     info!("end sel spawn");
        //     tokio::spawn(end_sel);
        //     Ok(())
        // }).map_err(|_| ());
        // tokio::spawn(sel);
    }

    pub fn delete_line(&mut self) {
        // FIXME: Find non blocking way by chaining
        self.inner.line_start(self.view_id).wait().unwrap();
        self.inner.line_end_sel(self.view_id).wait().unwrap();
        self.inner.delete(self.view_id).wait().unwrap();
    }

    pub fn goto_line(&mut self, line: u64) {
        let f = self.inner.goto_line(self.view_id, line).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn copy(&mut self) -> ClientResult<Value> {
        self.inner.copy(self.view_id)
    }

    pub fn paste(&mut self, buffer: &str) {
        let f = self.inner.paste(self.view_id, buffer).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn cut(&mut self) -> ClientResult<Value> {
        self.inner.cut(self.view_id)
    }

    pub fn undo(&mut self) {
        let f = self.inner.undo(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn redo(&mut self) {
        let f = self.inner.redo(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn find(
        &mut self,
        search_term: &str,
        case_sensitive: bool,
        regex: bool,
        whole_words: bool,
    ) {
        let f = self
            .inner
            .find(
                self.view_id,
                search_term,
                case_sensitive,
                regex,
                whole_words,
            )
            .map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn find_next(
        &mut self,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) {
        let f = self
            .inner
            .find_next(self.view_id, wrap_around, allow_same, modify_selection)
            .map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn find_prev(
        &mut self,
        wrap_around: bool,
        allow_same: bool,
        modify_selection: ModifySelection,
    ) {
        let f = self
            .inner
            .find_prev(self.view_id, wrap_around, allow_same, modify_selection)
            .map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn find_all(&mut self) {
        let f = self.inner.find_all(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn highlight_find(&mut self, visible: bool) {
        let f = self
            .inner
            .highlight_find(self.view_id, visible)
            .map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn down(&mut self) {
        let f = self.inner.down(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn up(&mut self) {
        let f = self.inner.up(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn right(&mut self) {
        let f = self.inner.right(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn left(&mut self) {
        let f = self.inner.left(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn page_down(&mut self) {
        let f = self.inner.page_down(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn page_up(&mut self) {
        let f = self.inner.page_up(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn home(&mut self) {
        let f = self.inner.line_start(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn end(&mut self) {
        let f = self.inner.line_end(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn delete(&mut self) {
        let f = self.inner.delete(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn backspace(&mut self) {
        let f = self.inner.backspace(self.view_id).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn save(&mut self, file: &str) -> ClientResult<()> {
        self.inner.save(self.view_id, file)
    }

    pub fn click(&mut self, line: u64, column: u64) {
        let f = self.inner.click(self.view_id, line, column).map_err(|_| ());
        tokio::spawn(f);
    }

    pub fn drag(&mut self, line: u64, column: u64) {
        let f = self.inner.drag(self.view_id, line, column).map_err(|_| ());
        tokio::spawn(f);
    }
}
