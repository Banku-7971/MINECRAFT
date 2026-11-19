//! Multiplayer over TCP. std::net uses Winsock on Windows and BSD sockets on
//! Linux, so the same code works for both without extra bindings.
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 25565;

pub enum NetEvent {
    Status(String),
    Line(String),
    Error(String),
    Closed,
}

enum Ctrl {
    Send(String),
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
                            // reader thread
                            let ev2 = ev_tx.clone();
                            let clients2 = clients.clone();
                            thread::spawn(move || {
                                let reader = BufReader::new(stream);
                                for line in reader.lines() {
                                    match line {
                                        Ok(l) => { let _ = ev2.send(NetEvent::Line(l)); }
                                        Err(_) => break,
                                    }
                                }
                                clients2.lock().unwrap().retain(|(cid, _)| *cid != id);
                                let _ = ev2.send(NetEvent::Status(format!("Player {} left", id)));
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
                let mut out: Vec<String> = Vec::new();
                loop {
                    match ctrl_rx.recv_timeout(Duration::from_millis(16)) {
                        Ok(Ctrl::Send(line)) => out.push(line),
                        Ok(Ctrl::Stop) => break,
                        Err(_) => {}
                    }
                    if out.is_empty() { continue; }
                    let list = clients.lock().unwrap();
                    for (_, tx) in list.iter() {
                        for l in out.iter() { let _ = tx.send(l.clone()); }
                    }
                    drop(list);
                    out.clear();
                }
            });
        }

        Net { ev: ev_rx, ctrl: Some(ctrl_tx), stop, hosting: true, connected: true }
    }

    pub fn join(addr: &str) -> Net {
        let (ev_tx, ev_rx) = channel();
        let (ctrl_tx, ctrl_rx) = channel::<Ctrl>();
        let stop = Arc::new(AtomicBool::new(false));
        let target = if addr.trim().contains(':') { addr.trim().to_string() } else { format!("{}:{}", addr.trim(), DEFAULT_PORT) };
        {
            let ev_tx = ev_tx.clone();
            thread::spawn(move || {
                let sock_addr = match target.parse() {
                    Ok(a) => a,
                    Err(_) => {
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
                let _ = ev_tx.send(NetEvent::Status(format!("Connected to {}", target)));
                let mut writer = match stream.try_clone() {
                    Ok(w) => w,
                    Err(_) => return,
                };
                let ev_w = ev_tx.clone();
                thread::spawn(move || {
                    while let Ok(Ctrl::Send(line)) = ctrl_rx.recv() {
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
