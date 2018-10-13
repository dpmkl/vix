use futures::Future;
use tokio;
use xrl;
use xrl::ClientResult;

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
