//! Multiplayer over TCP. std::net uses Winsock on Windows and BSD sockets on
//! Linux, so the same code works for both without extra bindings.
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 25565;

pub enum NetEvent {
    Status(String),
    Connected,
    ClientJoined(usize),
    Line(String),
    Error(String),
    Closed,
}

enum Ctrl {
    Send(String),
    SendTo(usize, String),
    Stop,
}

type Clients = Arc<Mutex<Vec<(usize, Sender<String>)>>>;

pub struct Net {
    ev: Receiver<NetEvent>,
    ctrl: Option<Sender<Ctrl>>,
    stop: Arc<AtomicBool>,
    pub hosting: bool,
    pub connected: bool,
}

impl Net {
    pub fn offline() -> Net {
        let (_tx, rx) = channel();
        Net { ev: rx, ctrl: None, stop: Arc::new(AtomicBool::new(false)), hosting: false, connected: false }
    }

    pub fn host(port: u16) -> Net {
        let (ev_tx, ev_rx) = channel();
        let (ctrl_tx, ctrl_rx) = channel::<Ctrl>();
        let stop = Arc::new(AtomicBool::new(false));
        let clients: Clients = Arc::new(Mutex::new(Vec::new()));

        // --- accept loop ---
        {
            let ev_tx = ev_tx.clone();
            let stop = stop.clone();
            let clients = clients.clone();
            thread::spawn(move || {
                let listener = match TcpListener::bind(("0.0.0.0", port)) {
                    Ok(l) => l,
                    Err(e) => {
                        let _ = ev_tx.send(NetEvent::Error(format!("Could not open port {}: {}", port, e)));
                        return;
                    }
                };
                let _ = listener.set_nonblocking(true);
                let _ = ev_tx.send(NetEvent::Status(format!("LAN world open on port {}", port)));
                let mut next_id = 1usize;
                loop {
                    if stop.load(Ordering::Relaxed) { break; }
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            let id = next_id;
                            next_id += 1;
                            let _ = stream.set_nodelay(true);
                            let (line_tx, line_rx) = channel::<String>();
                            clients.lock().unwrap().push((id, line_tx));
                            let _ = ev_tx.send(NetEvent::Status(format!("Player {} joined ({})", id, addr)));
                            let _ = ev_tx.send(NetEvent::ClientJoined(id));

                            // writer thread
                            if let Ok(mut w) = stream.try_clone() {
                                thread::spawn(move || {
                                    while let Ok(line) = line_rx.recv() {
                                        if w.write_all(line.as_bytes()).is_err() { break; }
                                        if w.write_all(b"\n").is_err() { break; }
                                        let _ = w.flush();
                                    }
                                });
                            }
                            // reader thread: forward to host AND relay to all other clients
                            let ev2 = ev_tx.clone();
                            let clients2 = clients.clone();
                            thread::spawn(move || {
                                let reader = BufReader::new(stream);
                                for line in reader.lines() {
                                    match line {
                                        Ok(l) => {
                                            let tagged = if let Some(rest) = l.strip_prefix("POS|") {
                                                format!("RPOS|{}|{}", id, rest)
                                            } else {
                                                l.clone()
                                            };
                                            let _ = ev2.send(NetEvent::Line(tagged.clone()));
                                            if let Ok(list) = clients2.lock() {
                                                for (cid, tx) in list.iter() {
                                                    if *cid != id {
                                                        let _ = tx.send(tagged.clone());
                                                    }
                                                }
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                                clients2.lock().unwrap().retain(|(cid, _)| *cid != id);
                                let _ = ev2.send(NetEvent::Status(format!("Player {} left", id)));
                                let _ = ev2.send(NetEvent::Line(format!("RLEAVE|{}", id)));
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(40));
                        }
                        Err(_) => thread::sleep(Duration::from_millis(80)),
                    }
                }
            });
        }

        // --- broadcast pump ---
        {
            let clients = clients.clone();
            thread::spawn(move || {
                loop {
                    match ctrl_rx.recv_timeout(Duration::from_millis(16)) {
                        Ok(Ctrl::Send(line)) => {
                            if let Ok(list) = clients.lock() {
                                for (_, tx) in list.iter() {
                                    let _ = tx.send(line.clone());
                                }
                            }
                        }
                        Ok(Ctrl::SendTo(target_id, line)) => {
                            if let Ok(list) = clients.lock() {
                                for (cid, tx) in list.iter() {
                                    if *cid == target_id {
                                        let _ = tx.send(line.clone());
                                    }
                                }
                            }
                        }
                        Ok(Ctrl::Stop) => break,
                        Err(_) => {}
                    }
                }
            });
        }

        Net { ev: ev_rx, ctrl: Some(ctrl_tx), stop, hosting: true, connected: true }
    }

    pub fn join(addr: &str) -> Net {
        let (ev_tx, ev_rx) = channel();
        let (ctrl_tx, ctrl_rx) = channel::<Ctrl>();
        let stop = Arc::new(AtomicBool::new(false));
        let raw = addr.trim();
        let target = if raw.is_empty() {
            format!("127.0.0.1:{}", DEFAULT_PORT)
        } else if raw.contains(':') {
            raw.to_string()
        } else {
            format!("{}:{}", raw, DEFAULT_PORT)
        };
        {
            let ev_tx = ev_tx.clone();
            thread::spawn(move || {
                let sock_addr = match target.to_socket_addrs().ok().and_then(|mut it| it.next()) {
                    Some(a) => a,
                    None => {
                        let _ = ev_tx.send(NetEvent::Error(format!("Bad address: {}", target)));
                        return;
                    }
                };
                let stream = match TcpStream::connect_timeout(&sock_addr, Duration::from_secs(6)) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = ev_tx.send(NetEvent::Error(format!("Could not connect: {}", e)));
                        return;
                    }
                };
                let _ = stream.set_nodelay(true);
                let _ = ev_tx.send(NetEvent::Connected);
                let _ = ev_tx.send(NetEvent::Status(format!("Connected to {}", target)));
                let mut writer = match stream.try_clone() {
                    Ok(w) => w,
                    Err(_) => return,
                };
                let ev_w = ev_tx.clone();
                thread::spawn(move || {
                    while let Ok(msg) = ctrl_rx.recv() {
                        let line = match msg {
                            Ctrl::Send(l) | Ctrl::SendTo(_, l) => l,
                            Ctrl::Stop => break,
                        };
                        if writer.write_all(line.as_bytes()).is_err() { break; }
                        if writer.write_all(b"\n").is_err() { break; }
                        let _ = writer.flush();
                    }
                    let _ = ev_w.send(NetEvent::Closed);
                });
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    match line {
                        Ok(l) => { let _ = ev_tx.send(NetEvent::Line(l)); }
                        Err(_) => break,
                    }
                }
                let _ = ev_tx.send(NetEvent::Closed);
            });
        }
        Net { ev: ev_rx, ctrl: Some(ctrl_tx), stop, hosting: false, connected: false }
    }

    pub fn send(&self, line: String) {
        if let Some(c) = &self.ctrl { let _ = c.send(Ctrl::Send(line)); }
    }

    pub fn send_to(&self, id: usize, line: String) {
        if let Some(c) = &self.ctrl { let _ = c.send(Ctrl::SendTo(id, line)); }
    }

    pub fn poll(&self) -> Vec<NetEvent> {
        let mut out = Vec::new();
        while let Ok(e) = self.ev.try_recv() { out.push(e); }
        out
    }

    pub fn close(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(c) = &self.ctrl { let _ = c.send(Ctrl::Stop); }
        self.ctrl = None;
        self.connected = false;
        self.hosting = false;
    }
}
