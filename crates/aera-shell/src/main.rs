#![forbid(unsafe_code)]

use aera_shell::{search, Desktop, Panel};
use eframe::egui::{self, Color32, RichText};
use std::path::{Path, PathBuf};

struct Shell {
    desktop: Desktop,
    query: String,
    dark: bool,
    zoom: f32,
    directory: Option<PathBuf>,
    entries: Vec<String>,
    file_error: Option<String>,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            desktop: Desktop::default(),
            query: String::new(),
            dark: true,
            zoom: 1.0,
            directory: std::env::current_dir().ok(),
            entries: Vec::new(),
            file_error: None,
        }
    }
}

fn read_directory(path: &Path) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(path)?.take(200) {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort_unstable();
    Ok(names)
}

impl Shell {
    fn launch(&mut self, panel: Panel) {
        self.desktop.set_open(panel, true);
        if panel == Panel::Files {
            self.refresh_files();
        }
    }

    fn refresh_files(&mut self) {
        self.entries.clear();
        self.file_error = None;
        match self.directory.as_deref().map(read_directory) {
            Some(Ok(entries)) => self.entries = entries,
            Some(Err(error)) => self.file_error = Some(error.to_string()),
            None => self.file_error = Some("Working directory unavailable".into()),
        }
    }

    fn panel(&mut self, ctx: &egui::Context, panel: Panel) {
        let mut open = self.desktop.is_open(panel);
        if !open {
            return;
        }
        let offset = panel as usize as f32 * 30.0;
        egui::Window::new(panel.title())
            .open(&mut open)
            .default_pos([80.0 + offset, 110.0 + offset])
            .default_size([380.0, 260.0])
            .resizable(true)
            .show(ctx, |ui| match panel {
                Panel::Files => {
                    if let Some(path) = &self.directory {
                        ui.label(path.display().to_string());
                    }
                    ui.label("Read-only working directory · first 200 entries");
                    if ui.button("Refresh").clicked() {
                        self.refresh_files();
                    }
                    if let Some(error) = &self.file_error {
                        ui.colored_label(Color32::LIGHT_RED, error);
                    }
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for name in &self.entries {
                            ui.label(name);
                        }
                    });
                }
                Panel::Settings => {
                    if ui.checkbox(&mut self.dark, "Dark appearance").changed() {
                        ctx.set_visuals(if self.dark {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        });
                    }
                    if ui
                        .add(egui::Slider::new(&mut self.zoom, 0.8..=1.6).text("UI zoom"))
                        .changed()
                    {
                        ctx.set_zoom_factor(self.zoom);
                    }
                    ui.label("Session settings only");
                }
                Panel::About => {
                    let target = aera_core::Target::current();
                    ui.heading("AERA");
                    ui.label("One OS. Any machine.");
                    ui.label(format!("{} / {}", target.os, target.arch));
                    ui.label("Hosted prototype · wgpu requested");
                    ui.label("Hardware and compositor support not qualified.");
                }
            });
        self.desktop.set_open(panel, open);
    }
}

impl eframe::App for Shell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("A E R A");
                ui.separator();
                ui.label("Workspace 01");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Hosted session");
                });
            });
        });
        egui::TopBottomPanel::bottom("dock").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                for panel in Panel::ALL {
                    if ui.button(panel.title()).clicked() {
                        self.launch(panel);
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(32.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("A E R A").size(40.0).strong());
                ui.label("One OS. Any machine.");
                ui.add_space(20.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search applications"),
                );
                let matches = search(&self.query);
                if response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    if let Some(panel) = matches.first() {
                        self.launch(*panel);
                    }
                }
                if !self.query.trim().is_empty() {
                    if matches.is_empty() {
                        ui.label("No applications found");
                    }
                    for panel in matches {
                        if ui.button(panel.title()).clicked() {
                            self.launch(panel);
                        }
                    }
                }
            });
        });
        for panel in Panel::ALL {
            self.panel(ctx, panel);
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AERA Shell",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(Shell::default()))
        }),
    )
}
