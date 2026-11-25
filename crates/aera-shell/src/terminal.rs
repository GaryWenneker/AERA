use eframe::egui;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;

const LIMIT: usize = 256 * 1024;

pub struct Terminal {
    input: String,
    output: String,
    cwd: PathBuf,
    result: Option<mpsc::Receiver<String>>,
    cancel: Arc<AtomicBool>,
    history: Vec<String>,
    cursor: usize,
}

impl Default for Terminal {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            result: None,
            cancel: Arc::new(AtomicBool::new(false)),
            history: Vec::new(),
            cursor: 0,
        }
    }
}

fn drain<R: Read>(mut stream: R) -> Vec<u8> {
    let mut data = Vec::new();
    let mut buf = [0u8; 4096];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 { break; }
        let keep = n.min(LIMIT.saturating_sub(data.len()));
        data.extend_from_slice(&buf[..keep]);
    }
    data
}

impl Terminal {
    fn append(&mut self, text: &str) {
        self.output.push_str(text);
        if self.output.len() > LIMIT {
            let mut start = self.output.len() - LIMIT;
            while !self.output.is_char_boundary(start) { start += 1; }
            self.output.drain(..start);
        }
    }

    fn submit(&mut self, ctx: &egui::Context) {
        let command = std::mem::take(&mut self.input);
        let command = command.trim().to_owned();
        if command.is_empty() { return; }
        self.history.push(command.clone());
        if self.history.len() > 100 { self.history.remove(0); }
        self.cursor = self.history.len();
        self.append(&format!("$ {command}\n"));
        if command == "clear" {
            self.output.clear();
            return;
        }
        if command == "cd" || command.starts_with("cd ") {
            let target = if command == "cd" {
                super::home().unwrap_or_else(|| self.cwd.clone())
            } else {
                self.cwd.join(command[3..].trim().trim_matches('"'))
            };
            match target.canonicalize() {
                Ok(path) if path.is_dir() => self.cwd = path,
                _ => self.append("cd: directory unavailable\n"),
            }
            return;
        }
        let cwd = self.cwd.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = cancel.clone();
        let (tx, rx) = mpsc::channel();
        self.result = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut process = if cfg!(windows) {
                let mut c = Command::new("cmd.exe");
                c.args(["/D", "/C", &command]);
                c
            } else {
                let mut c = Command::new("/bin/sh");
                c.args(["-c", &command]);
                c
            };
            let result = process.current_dir(cwd).stdin(Stdio::null())
                .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn();
            let message = match result {
                Ok(mut child) => {
                    let stdout = child.stdout.take().expect("piped stdout");
                    let stderr = child.stderr.take().expect("piped stderr");
                    let out = std::thread::spawn(move || drain(stdout));
                    let err = std::thread::spawn(move || drain(stderr));
                    let status = loop {
                        if cancel.load(Ordering::Relaxed) {
                            let _ = child.kill();
                            break child.wait();
                        }
                        match child.try_wait() {
                            Ok(Some(status)) => break Ok(status),
                            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
                            Err(error) => break Err(error),
                        }
                    };
                    // Descendants may retain pipes; do not block the UI waiting for them.
                    let deadline = std::time::Instant::now() + Duration::from_secs(1);
                    while !(out.is_finished() && err.is_finished()) && std::time::Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    let a = if out.is_finished() { out.join().unwrap_or_default() } else { Vec::new() };
                    let b = if err.is_finished() { err.join().unwrap_or_default() } else { Vec::new() };
                    format!("{}{}\n[{:?}]\n", String::from_utf8_lossy(&a), String::from_utf8_lossy(&b), status)
                }
                Err(error) => format!("Launch failed: {error}\n"),
            };
            let _ = tx.send(message);
            ctx.request_repaint();
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, guest: bool) {
        if let Some(rx) = &self.result {
            match rx.try_recv() {
                Ok(message) => { self.result = None; self.append(&message); }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.result = None;
                    self.append("Command worker stopped\n");
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        ui.small(if guest { "Guest shell" } else { "Host shell · commands run on this computer" });
        ui.monospace(self.cwd.display().to_string());
        egui::ScrollArea::vertical().stick_to_bottom(true)
            .max_height((ui.available_height() - 54.0).max(60.0))
            .show(ui, |ui| { ui.monospace(&self.output); });
        ui.horizontal(|ui| {
            ui.label("$");
            let input = ui.add_enabled(self.result.is_none(), egui::TextEdit::singleline(&mut self.input).font(egui::TextStyle::Monospace).desired_width((ui.available_width()-85.0).max(60.0)));
            if input.has_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.cursor = self.cursor.saturating_sub(1);
                    if let Some(value) = self.history.get(self.cursor) { self.input = value.clone(); }
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.cursor = (self.cursor + 1).min(self.history.len());
                    self.input = self.history.get(self.cursor).cloned().unwrap_or_default();
                }
            }
            let enter = (input.has_focus() || input.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if self.result.is_none() {
                if ui.button("Run").clicked() || enter { self.submit(ui.ctx()); input.request_focus(); }
            } else if ui.button("Stop").clicked() {
                self.cancel.store(true, Ordering::Relaxed);
            }
        });
    }
}
