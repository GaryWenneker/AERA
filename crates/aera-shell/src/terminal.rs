use eframe::egui;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::Duration;

const LIMIT: usize = 256 * 1024;

pub struct Terminal {
    input: String,
    output: String,
    cwd: PathBuf,
    result: Option<mpsc::Receiver<Event>>,
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

enum Event {
    Output(String),
    Done(String),
}

fn stream<R: Read>(mut reader: R, tx: mpsc::SyncSender<Event>, ctx: egui::Context) {
    let mut buf = [0u8; 4096];
    let mut pending = Vec::new();
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        pending.extend_from_slice(&buf[..n]);
        let end = match std::str::from_utf8(&pending) {
            Ok(_) => pending.len(),
            Err(e) if e.error_len().is_none() => e.valid_up_to(),
            Err(_) => pending.len(),
        };
        if end > 0 {
            let text = String::from_utf8_lossy(&pending[..end]).into_owned();
            pending.drain(..end);
            if tx.send(Event::Output(text)).is_err() {
                return;
            }
            ctx.request_repaint();
        }
    }
    if !pending.is_empty() {
        let _ = tx.send(Event::Output(
            String::from_utf8_lossy(&pending).into_owned(),
        ));
        ctx.request_repaint();
    }
}

impl Terminal {
    fn append(&mut self, text: &str) {
        self.output.push_str(text);
        if self.output.len() > LIMIT {
            let mut start = self.output.len() - LIMIT;
            while !self.output.is_char_boundary(start) {
                start += 1;
            }
            self.output.drain(..start);
        }
    }

    fn submit(&mut self, ctx: &egui::Context) {
        let command = std::mem::take(&mut self.input);
        let command = command.trim().to_owned();
        if command.is_empty() {
            return;
        }
        self.history.push(command.clone());
        if self.history.len() > 100 {
            self.history.remove(0);
        }
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
        let (tx, rx) = mpsc::sync_channel(64);
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
            let result = process
                .current_dir(cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            let message = match result {
                Ok(mut child) => {
                    let stdout = child.stdout.take().expect("piped stdout");
                    let stderr = child.stderr.take().expect("piped stderr");
                    let out_tx = tx.clone();
                    let err_tx = tx.clone();
                    let out_ctx = ctx.clone();
                    let err_ctx = ctx.clone();
                    let out = std::thread::spawn(move || stream(stdout, out_tx, out_ctx));
                    let err = std::thread::spawn(move || stream(stderr, err_tx, err_ctx));
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
                    while !(out.is_finished() && err.is_finished())
                        && std::time::Instant::now() < deadline
                    {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    if out.is_finished() {
                        let _ = out.join();
                    }
                    if err.is_finished() {
                        let _ = err.join();
                    }
                    format!("\n[{status:?}]\n")
                }
                Err(error) => format!("Launch failed: {error}\n"),
            };
            let _ = tx.send(Event::Done(message));
            ctx.request_repaint();
        });
    }

    pub fn poll(&mut self, ctx: &egui::Context) {
        for _ in 0..64 {
            let event = match &self.result {
                Some(rx) => rx.try_recv(),
                None => break,
            };
            match event {
                Ok(Event::Output(text)) => self.append(&text),
                Ok(Event::Done(message)) => {
                    self.result = None;
                    self.append(&message);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.result = None;
                    self.append("Command worker stopped\n");
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
        if self.result.is_some() {
            ctx.request_repaint_after(Duration::from_millis(40));
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, guest: bool) {
        self.poll(ui.ctx());
        ui.small(if guest {
            "Guest shell"
        } else {
            "Host shell · commands run on this computer"
        });
        ui.monospace(self.cwd.display().to_string());
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height((ui.available_height() - 54.0).max(60.0))
            .show(ui, |ui| {
                ui.monospace(&self.output);
            });
        ui.horizontal(|ui| {
            ui.label("$");
            let input = ui.add_enabled(
                self.result.is_none(),
                egui::TextEdit::singleline(&mut self.input)
                    .font(egui::TextStyle::Monospace)
                    .desired_width((ui.available_width() - 85.0).max(60.0)),
            );
            if input.has_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.cursor = self.cursor.saturating_sub(1);
                    if let Some(value) = self.history.get(self.cursor) {
                        self.input = value.clone();
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.cursor = (self.cursor + 1).min(self.history.len());
                    self.input = self.history.get(self.cursor).cloned().unwrap_or_default();
                }
            }
            let enter = (input.has_focus() || input.lost_focus())
                && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if self.result.is_none() {
                if ui.button("Run").clicked() || enter {
                    self.submit(ui.ctx());
                    input.request_focus();
                }
            } else if ui.button("Stop").clicked() {
                self.cancel.store(true, Ordering::Relaxed);
            }
        });
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_bounded_and_preserves_utf8() {
        let mut terminal = Terminal::default();
        terminal.append(&"é".repeat(LIMIT));
        assert!(terminal.output.len() <= LIMIT);
        assert!(terminal.output.chars().all(|c| c == 'é'));
    }

    #[test]
    fn stream_preserves_split_utf8() {
        struct ByteReader(std::io::Cursor<Vec<u8>>);
        impl Read for ByteReader {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let count = buf.len().min(1);
                self.0.read(&mut buf[..count])
            }
        }
        let (tx, rx) = mpsc::sync_channel(64);
        stream(
            ByteReader(std::io::Cursor::new("Aé界".as_bytes().to_vec())),
            tx,
            egui::Context::default(),
        );
        let output: String = rx
            .try_iter()
            .filter_map(|event| match event {
                Event::Output(text) => Some(text),
                Event::Done(_) => None,
            })
            .collect();
        assert_eq!(output, "Aé界");
    }

    #[test]
    fn live_output_does_not_end_session() {
        let mut terminal = Terminal::default();
        let (tx, rx) = mpsc::sync_channel(4);
        terminal.result = Some(rx);
        let ctx = egui::Context::default();
        tx.send(Event::Output("first".into())).unwrap();
        terminal.poll(&ctx);
        assert_eq!(terminal.output, "first");
        assert!(terminal.result.is_some());
        tx.send(Event::Done("done".into())).unwrap();
        terminal.poll(&ctx);
        assert!(terminal.result.is_none());
        assert_eq!(terminal.output, "firstdone");
    }
}
