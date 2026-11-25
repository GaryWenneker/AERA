//! Interactive PTY and VT screen, independent of the desktop renderer.
use eframe::egui::{self, Color32};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc;

pub struct Session {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    input: mpsc::SyncSender<Vec<u8>>,
    output: mpsc::Receiver<Vec<u8>>,
    parser: vt100::Parser,
    size: (u16, u16),
}

impl Session {
    pub fn start(ctx: &egui::Context) -> Result<Self, String> {
        let size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = native_pty_system()
            .openpty(size)
            .map_err(|e| e.to_string())?;
        let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
        let mut writer = pair.master.take_writer().map_err(|e| e.to_string())?;
        let mut cmd = CommandBuilder::new_default_prog();
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
        drop(pair.slave);
        let (tx, output) = mpsc::sync_channel(64);
        let (input, rx) = mpsc::sync_channel::<Vec<u8>>(32);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
            ctx.request_repaint();
        });
        std::thread::spawn(move || {
            while let Ok(bytes) = rx.recv() {
                if writer
                    .write_all(&bytes)
                    .and_then(|_| writer.flush())
                    .is_err()
                {
                    break;
                }
            }
        });
        Ok(Self {
            master: pair.master,
            child,
            input,
            output,
            parser: vt100::Parser::new(24, 80, 2000),
            size: (24, 80),
        })
    }

    pub fn poll(&mut self) {
        for _ in 0..64 {
            match self.output.try_recv() {
                Ok(bytes) => self.parser.process(&bytes),
                Err(_) => break,
            }
        }
    }

    fn send(&self, bytes: Vec<u8>) -> bool {
        self.input.try_send(bytes).is_ok()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll();
        let font = egui::FontId::monospace(13.0);
        let width = ui.fonts(|f| f.glyph_width(&font, 'M')).max(1.0);
        let height = ui.fonts(|f| f.row_height(&font)).max(1.0);
        let avail = ui.available_size().max(egui::vec2(40.0, 30.0));
        let cols = (avail.x / width).floor().clamp(2.0, 240.0) as u16;
        let rows = (avail.y / height).floor().clamp(2.0, 100.0) as u16;
        if self.size != (rows, cols)
            && self
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .is_ok()
        {
            self.parser.set_size(rows, cols);
            self.size = (rows, cols);
        }
        let (rect, response) = ui.allocate_exact_size(avail, egui::Sense::click());
        if response.clicked() {
            response.request_focus();
        }
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6, Color32::from_rgb(19, 23, 32));
        let fg = Color32::from_rgb(225, 231, 240);
        let bg = Color32::from_rgb(19, 23, 32);
        let screen = self.parser.screen();
        for row in 0..self.size.0 {
            for col in 0..self.size.1 {
                let Some(cell) = screen.cell(row, col) else {
                    continue;
                };
                let pos = rect.min + egui::vec2(col as f32 * width, row as f32 * height);
                let mut foreground = color(cell.fgcolor(), fg);
                let mut background = color(cell.bgcolor(), bg);
                if cell.inverse() {
                    std::mem::swap(&mut foreground, &mut background);
                }
                if background != bg {
                    painter.rect_filled(
                        egui::Rect::from_min_size(pos, egui::vec2(width, height)),
                        0,
                        background,
                    );
                }
                if cell.has_contents() {
                    painter.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        cell.contents(),
                        font.clone(),
                        foreground,
                    );
                }
            }
        }
        if !screen.hide_cursor() {
            let (r, c) = screen.cursor_position();
            let pos = rect.min + egui::vec2(c as f32 * width, r as f32 * height);
            painter.rect_stroke(
                egui::Rect::from_min_size(pos, egui::vec2(width, height)),
                0,
                egui::Stroke::new(1.0_f32, fg),
                egui::StrokeKind::Inside,
            );
        }
        if response.has_focus() {
            let events = ui.input(|i| i.events.clone());
            for event in events {
                let bytes = match event {
                    egui::Event::Text(text) | egui::Event::Paste(text) => text.into_bytes(),
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        let sequence = match key {
                            egui::Key::Enter => "\r",
                            egui::Key::Backspace => "\x7f",
                            egui::Key::Tab => "\t",
                            egui::Key::Escape => "\x1b",
                            egui::Key::ArrowUp => "\x1b[A",
                            egui::Key::ArrowDown => "\x1b[B",
                            egui::Key::ArrowRight => "\x1b[C",
                            egui::Key::ArrowLeft => "\x1b[D",
                            egui::Key::Home => "\x1b[H",
                            egui::Key::End => "\x1b[F",
                            egui::Key::Delete => "\x1b[3~",
                            egui::Key::C if modifiers.ctrl => "\x03",
                            egui::Key::D if modifiers.ctrl => "\x04",
                            egui::Key::L if modifiers.ctrl => "\x0c",
                            _ => "",
                        };
                        sequence.as_bytes().to_vec()
                    }
                    _ => Vec::new(),
                };
                if !bytes.is_empty() && !self.send(bytes) {
                    painter.text(
                        rect.right_top(),
                        egui::Align2::RIGHT_TOP,
                        "Input queue full / shell closed",
                        font.clone(),
                        Color32::LIGHT_RED,
                    );
                }
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn color(value: vt100::Color, default: Color32) -> Color32 {
    match value {
        vt100::Color::Default => default,
        vt100::Color::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
        vt100::Color::Idx(i) => {
            const BASE: [[u8; 3]; 16] = [
                [0, 0, 0],
                [205, 75, 80],
                [110, 190, 125],
                [225, 185, 100],
                [105, 145, 230],
                [180, 130, 215],
                [100, 195, 205],
                [220, 225, 230],
                [110, 115, 125],
                [245, 110, 115],
                [150, 225, 160],
                [250, 220, 145],
                [155, 185, 255],
                [220, 175, 250],
                [150, 230, 240],
                [255, 255, 255],
            ];
            if i < 16 {
                let [r, g, b] = BASE[i as usize];
                Color32::from_rgb(r, g, b)
            } else if i < 232 {
                let n = i - 16;
                let component = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
                Color32::from_rgb(component(n / 36), component(n / 6 % 6), component(n % 6))
            } else {
                Color32::from_gray(8 + 10 * (i - 232))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn vt_grid_interprets_cursor_and_color() {
        let mut parser = vt100::Parser::new(4, 20, 10);
        parser.process(b"abc\x1b[2;3H\x1b[31mX");
        let cell = parser.screen().cell(1, 2).unwrap();
        assert_eq!(cell.contents(), "X");
        assert_eq!(cell.fgcolor(), vt100::Color::Idx(1));
    }
}
