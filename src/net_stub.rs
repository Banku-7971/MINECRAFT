//! Browser build replacement for the TCP multiplayer module.
//!
//! Browsers cannot open raw TCP sockets, so the WebAssembly build runs in
//! singleplayer and reports a friendly message if someone opens the LAN menu.

use std::cell::RefCell;

pub const DEFAULT_PORT: u16 = 25565;

pub enum NetEvent {
    Status(String),
    Connected,
    ClientJoined(usize),
    Line(String),
    Error(String),
    Closed,
}

pub struct Net {
    pending: RefCell<Vec<NetEvent>>,
    pub hosting: bool,
    pub connected: bool,
}

impl Net {
    pub fn offline() -> Net {
        Net { pending: RefCell::new(Vec::new()), hosting: false, connected: false }
    }

    pub fn host(_port: u16) -> Net {
        Net {
            pending: RefCell::new(vec![NetEvent::Status(
                "LAN multiplayer is not available in the browser build.".to_string(),
            )]),
            hosting: false,
            connected: false,
        }
    }

    pub fn join(_addr: &str) -> Net {
        Net {
            pending: RefCell::new(vec![NetEvent::Status(
                "LAN multiplayer is not available in the browser build.".to_string(),
            )]),
            hosting: false,
            connected: false,
        }
    }

    pub fn send(&self, _line: String) {}

    pub fn send_to(&self, _id: usize, _line: String) {}

    pub fn poll(&self) -> Vec<NetEvent> {
        self.pending.borrow_mut().drain(..).collect()
    }

    pub fn close(&mut self) {
        self.pending.get_mut().clear();
        self.hosting = false;
        self.connected = false;
    }
}
