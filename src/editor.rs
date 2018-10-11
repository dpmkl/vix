use crate::client::Client as ViewClient;
use crate::tui::CoreEvent;
use crate::view::View;
use crate::window::Window;
use futures::sync::mpsc::UnboundedReceiver;
use futures::{Async, Future, Stream};
use std::collections::HashMap;
use std::io::Write;
use termion::event::Event;
use tokio;
use xrl::{Client, ClientResult, ScrollTo, Style, Update, ViewId};

pub struct Editor {
    pub pending_open_requests: Vec<ClientResult<(ViewId, View)>>,
    pub delayed_events: Vec<CoreEvent>,
    pub views: HashMap<ViewId, View>,
    pub current_view: ViewId,
    pub events: UnboundedReceiver<CoreEvent>,
    pub client: Client,
    pub size: (u16, u16),
    pub styles: HashMap<u64, Style>,
}

impl Editor {
    pub fn new(client: Client, events: UnboundedReceiver<CoreEvent>) -> Editor {
        let mut styles = HashMap::new();
        styles.insert(0, Default::default());

        Editor {
            events,
            delayed_events: Vec::new(),
            pending_open_requests: Vec::new(),
            size: (0, 0),
            views: HashMap::new(),
            styles,
            current_view: ViewId(0),
            client,
        }
    }
}

impl Editor {
    pub fn dispatch_core_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::Update(update) => self.handle_update(update),
            CoreEvent::SetStyle(style) => self.handle_def_style(style),
            CoreEvent::ScrollTo(scroll_to) => self.handle_scroll_to(scroll_to),
        }
    }

    fn handle_update(&mut self, update: Update) {
        match self.views.get_mut(&update.view_id) {
            Some(view) => view.update_cache(update),
            None => self.delayed_events.push(CoreEvent::Update(update)),
        }
    }

    fn handle_scroll_to(&mut self, scroll_to: ScrollTo) {
        match self.views.get_mut(&scroll_to.view_id) {
            Some(view) => view.set_cursor(scroll_to.line, scroll_to.column),
            None => self.delayed_events.push(CoreEvent::ScrollTo(scroll_to)),
        }
    }

    fn handle_def_style(&mut self, style: Style) {
        self.styles.insert(style.id, style);
    }
}

impl Editor {
    pub fn open(&mut self, file_path: Option<String>) {
        let client = self.client.clone();
        let task = self
            .client
            .new_view(file_path.clone())
            .and_then(move |view_id| {
                let view_client = ViewClient::new(client, view_id);
                Ok((
                    view_id,
                    View::new(view_client, Some(file_path.unwrap_or_else(|| "".into()))),
                ))
            });
        self.pending_open_requests.push(Box::new(task));
    }

    pub fn set_theme(&mut self, theme: &str) {
        let future = self.client.set_theme(theme).map_err(|_| ());
        tokio::run(future);
    }

    pub fn save(&mut self, view: Option<ViewId>) {
        match view {
            Some(view_id) => {
                if let Some(view) = self.views.get_mut(&view_id) {
                    view.save();
                }
            }
            None => {
                if let Some(view) = self.views.get_mut(&self.current_view) {
                    view.save();
                }
            }
        }
    }
}

impl Editor {
    pub fn process_open_requests(&mut self) {
        if self.pending_open_requests.is_empty() {
            return;
        }

        info!("process pending open requests");

        let mut done = vec![];
        for (idx, task) in self.pending_open_requests.iter_mut().enumerate() {
            match task.poll() {
                Ok(Async::Ready((id, mut view))) => {
                    info!("open request succeeded for {}", &id);
                    done.push(idx);
                    view.resize(self.size.1);
                    self.views.insert(id, view);
                    self.current_view = id;
                }
                Ok(Async::NotReady) => continue,
                Err(e) => panic!("\"open\" task failed: {}", e),
            }
        }
        for idx in done.iter().rev() {
            self.pending_open_requests.remove(*idx);
        }

        if self.pending_open_requests.is_empty() {
            info!("no more pending open request");
        }
    }

    pub fn process_core_events(&mut self) {
        loop {
            match self.events.poll() {
                Ok(Async::Ready(Some(event))) => {
                    self.dispatch_core_event(event);
                }
                Ok(Async::Ready(None)) => {
                    error!("Error core stdout shut down => panicking");
                    panic!("Error core stdout shut down");
                }
                Ok(Async::NotReady) => break,
                Err(_) => {
                    error!("Error while polling core => panicking");
                    panic!("Error while polling core");
                }
            }
        }
    }

    pub fn process_delayed_events(&mut self) {
        let delayed_events: Vec<CoreEvent> = self.delayed_events.drain(..).collect();
        for event in delayed_events {
            self.dispatch_core_event(event);
        }
    }

    pub fn render<W: Write>(&mut self, term: &mut W)
    where
        W: std::fmt::Write,
    {
        if let Some(view) = self.views.get_mut(&self.current_view) {
            view.render(term, &self.styles);
        }
    }
}
