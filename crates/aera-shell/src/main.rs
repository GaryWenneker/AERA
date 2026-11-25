#![forbid(unsafe_code)]

use aera_shell::{search, Desktop, Panel, WindowState};
use eframe::egui::{self, Color32, RichText};
mod design;
mod disks;
mod files;
mod glass;

use std::path::PathBuf;

struct Shell {
    desktop: Desktop,
    query: String,
    dark: bool,
    glass_enabled: bool,
    glass_available: bool,
    glass_quality: f32,
    reduced_motion: bool,
    zoom: f32,
    directory: Option<PathBuf>,
    entries: Vec<files::Entry>,
    volumes: Vec<disks::Volume>,
    selected: Option<PathBuf>,
    file_error: Option<String>,
    file_name: String,
    file_filter: String,
    file_notice: Option<String>,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            desktop: Desktop::default(),
            query: String::new(),
            dark: true,
            glass_enabled: true,
            glass_available: false,
            glass_quality: 0.65,
            reduced_motion: false,
            zoom: 1.0,
            directory: std::env::current_dir().ok(),
            entries: Vec::new(),
            volumes: Vec::new(),
            selected: None,
            file_error: None,
            file_name: String::new(),
            file_filter: String::new(),
            file_notice: None,
        }
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
}

impl Shell {
    fn launch(&mut self, panel: Panel) {
        self.desktop.set_open(panel, true);
        if panel == Panel::Files {
            self.refresh_files();
        } else if panel == Panel::Disks {
            self.volumes = disks::inventory();
        }
    }

    fn refresh_files(&mut self) {
        self.entries.clear();
        self.selected = None;
        self.file_error = None;
        match self.directory.as_deref().map(files::list) {
            Some(Ok(entries)) => self.entries = entries,
            Some(Err(error)) => self.file_error = Some(error.to_string()),
            None => self.file_error = Some("Working directory unavailable".into()),
        }
    }

    fn navigate(&mut self, path: PathBuf) {
        match files::list(&path) {
            Ok(entries) => {
                self.directory = Some(path);
                self.entries = entries;
                self.selected = None;
                self.file_error = None;
            }
            Err(error) => self.file_error = Some(error.to_string()),
        }
    }

    fn file_action(&mut self, action: u8) {
        let Some(directory) = self.directory.as_deref() else {
            self.file_error = Some("No directory selected".into());
            return;
        };
        let result = match action {
            0 => files::create(directory, &self.file_name, true),
            1 => files::create(directory, &self.file_name, false),
            _ => match &self.selected {
                Some(source) => files::copy_file(source, directory, &self.file_name),
                None => return,
            },
        };
        self.file_notice = None;
        match result {
            Ok(()) => {
                self.file_name.clear();
                self.refresh_files();
                self.file_notice = Some("Operation completed".into());
            }
            Err(error) => {
                self.file_error = Some(format!("Operation failed: {error}"));
            }
        }
    }

    fn files_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Home").clicked() {
                if let Some(path) = home() {
                    self.navigate(path);
                }
            }
            if ui.button("Parent").clicked() {
                if let Some(path) = self.directory.as_deref().and_then(std::path::Path::parent) {
                    self.navigate(path.to_path_buf());
                }
            }
            if ui.button("Refresh").clicked() {
                self.refresh_files();
            }
            if ui.button("Volumes").clicked() {
                self.launch(Panel::Disks);
            }
        });
        if let Some(path) = &self.directory {
            ui.label(path.display().to_string());
        }
        if let Some(error) = &self.file_error {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.file_filter);
        });
        let mut next = None;
        egui::ScrollArea::vertical()
            .max_height(280.0)
            .show(ui, |ui| {
                egui::Grid::new("file-table").striped(true).show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Type");
                    ui.strong("Size (bytes)");
                    ui.end_row();
                    let filter = self.file_filter.to_lowercase();
                    for entry in &self.entries {
                        if !entry
                            .name
                            .to_string_lossy()
                            .to_lowercase()
                            .contains(&filter)
                        {
                            continue;
                        }
                        let response = ui.selectable_label(
                            self.selected.as_ref() == Some(&entry.path),
                            entry.name.to_string_lossy(),
                        );
                        if response.clicked() {
                            self.selected = Some(entry.path.clone());
                        }
                        if response.double_clicked() && entry.kind == "Folder" {
                            next = Some(entry.path.clone());
                        }
                        ui.label(entry.kind);
                        ui.label(entry.size.map_or_else(|| "-".into(), |n| n.to_string()));
                        ui.end_row();
                    }
                });
            });
        if let Some(path) = next {
            self.navigate(path);
        }
        ui.separator();
        ui.label(format!("{} items · first 1000 entries", self.entries.len()));
        if let Some(path) = &self.selected {
            ui.label(format!("Selected: {}", path.display()));
        }
        ui.separator();
        ui.label("Create or copy in the displayed directory (no overwrite)");
        ui.horizontal(|ui| {
            ui.label("New name");
            ui.text_edit_singleline(&mut self.file_name);
        });
        ui.horizontal_wrapped(|ui| {
            let ready = self.directory.is_some() && !self.file_name.is_empty();
            if ui
                .add_enabled(ready, egui::Button::new("New folder"))
                .clicked()
            {
                self.file_action(0);
            }
            if ui
                .add_enabled(ready, egui::Button::new("New file"))
                .clicked()
            {
                self.file_action(1);
            }
            let copyable = self
                .entries
                .iter()
                .any(|entry| entry.kind == "File" && self.selected.as_ref() == Some(&entry.path));
            if ui
                .add_enabled(ready && copyable, egui::Button::new("Copy selected"))
                .clicked()
            {
                self.file_action(2);
            }
        });
        if let Some(notice) = &self.file_notice {
            ui.label(notice);
        }
    }

    fn disks_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Mounted volumes");
        ui.label("Read-only capacity overview; not a physical partition inventory.");
        if ui.button("Refresh").clicked() {
            self.volumes = disks::inventory();
        }
        if self.volumes.is_empty() {
            ui.label("No volumes reported by this platform.");
        }
        let mut browse = None;
        egui::ScrollArea::vertical()
            .max_height(280.0)
            .show(ui, |ui| {
                for volume in &self.volumes {
                    ui.group(|ui| {
                        ui.strong(&volume.name);
                        ui.label(format!(
                            "{} · {}",
                            volume.mount.display(),
                            volume.filesystem
                        ));
                        if volume.total > 0 {
                            let used = disks::used_fraction(volume.total, volume.available);
                            ui.add(egui::ProgressBar::new(used).show_percentage());
                            ui.label(format!(
                                "{:.1} GiB available / {:.1} GiB total",
                                volume.available as f64 / 1073741824.0,
                                volume.total as f64 / 1073741824.0
                            ));
                        } else {
                            ui.label("Capacity unavailable");
                        }
                        if ui.button("Browse volume").clicked() {
                            browse = Some(volume.mount.clone());
                        }
                    });
                }
            });
        if let Some(path) = browse {
            self.desktop.set_open(Panel::Files, true);
            self.navigate(path);
        }
    }

    fn app_content(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, panel: Panel) {
        match panel {
            Panel::Files => self.files_view(ui),
            Panel::Disks => self.disks_view(ui),
            Panel::Settings => {
                if ui.checkbox(&mut self.dark, "Dark appearance").changed() {
                    design::apply(ctx, self.dark);
                }
                if ui
                    .add(egui::Slider::new(&mut self.zoom, 0.8..=1.6).text("UI zoom"))
                    .changed()
                {
                    ctx.set_zoom_factor(self.zoom);
                }
                ui.separator();
                ui.heading("Liquid Glass");
                ui.add_enabled(
                    self.glass_available,
                    egui::Checkbox::new(&mut self.glass_enabled, "Enable GPU glass"),
                );
                ui.add(egui::Slider::new(&mut self.glass_quality, 0.0..=1.0).text("Blur quality"));
                ui.checkbox(&mut self.reduced_motion, "Reduce motion");
                ui.label("Wallpaper-only optical preview; opaque application surfaces.");
                ui.separator();
                ui.heading("Storage & system");
                if ui.button("Open Disks").clicked() {
                    self.launch(Panel::Disks);
                }
                if ui.button("System information").clicked() {
                    self.launch(Panel::About);
                }
                ui.label("Session settings only");
            }
            Panel::About => {
                let target = aera_core::Target::current();
                ui.heading("AERA");
                ui.label("One OS. Any machine.");
                ui.label(format!("{} / {}", target.os, target.arch));
                ui.label(if guest_session() {
                    "Standalone guest · wgpu requested"
                } else {
                    "Hosted prototype · wgpu requested"
                });
                ui.label("Hardware and compositor support not qualified.");
            }
        }
    }

    fn window_controls(&mut self, ui: &mut egui::Ui, panel: Panel) {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("×").color(Color32::from_rgb(230, 95, 95)))
                .on_hover_text("Close window")
                .clicked()
            {
                self.desktop.set_state(panel, WindowState::Closed);
            }
            if ui
                .button(RichText::new("−").color(Color32::from_rgb(213, 163, 65)))
                .on_hover_text("Minimize window")
                .clicked()
            {
                self.desktop.set_state(panel, WindowState::Minimized);
            }
            let maximized = self.desktop.state(panel) == WindowState::Maximized;
            if ui
                .button(
                    RichText::new(if maximized { "↙" } else { "□" })
                        .color(Color32::from_rgb(75, 180, 135)),
                )
                .on_hover_text(if maximized {
                    "Restore window"
                } else {
                    "Maximize window"
                })
                .clicked()
            {
                self.desktop.set_state(
                    panel,
                    if maximized {
                        WindowState::Floating
                    } else {
                        WindowState::Maximized
                    },
                );
            }
            ui.strong(panel.title());
        });
        ui.separator();
    }

    fn panel(&mut self, ctx: &egui::Context, panel: Panel) {
        if self.desktop.state(panel) != WindowState::Floating {
            return;
        }
        let offset = panel as usize as f32 * 30.0;
        let workspace = self.desktop.active();
        egui::Window::new(panel.title())
            .id(egui::Id::new(("app", workspace, panel as usize)))
            .default_pos([80.0 + offset, 80.0 + offset])
            .default_size(if matches!(panel, Panel::Files | Panel::Disks) {
                [650.0, 380.0]
            } else {
                [380.0, 260.0]
            })
            .resizable(true)
            .show(ctx, |ui| {
                self.window_controls(ui, panel);
                self.app_content(ctx, ui, panel);
            });
    }
}

impl eframe::App for Shell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (previous, next, escape) = ctx.input(|input| {
            let shortcut = input.modifiers.ctrl && input.modifiers.alt;
            (
                shortcut && input.key_pressed(egui::Key::ArrowLeft),
                shortcut && input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::Escape),
            )
        });
        if previous {
            self.desktop.switch(self.desktop.active().saturating_sub(1));
        }
        if next {
            self.desktop.switch(self.desktop.active() + 1);
        }
        if escape {
            if let Some(panel) = self.desktop.maximized() {
                self.desktop.set_state(panel, WindowState::Floating);
            }
        }
        if let Some(panel) = self.desktop.maximized() {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.window_controls(ui, panel);
                self.app_content(ctx, ui, panel);
            });
            return;
        }
        egui::TopBottomPanel::top("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("A E R A", |ui| {
                    for panel in Panel::ALL {
                        if ui.button(panel.title()).clicked() {
                            self.launch(panel);
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                for index in 0..self.desktop.count() {
                    if ui
                        .selectable_label(self.desktop.active() == index, format!("{}", index + 1))
                        .clicked()
                    {
                        self.desktop.switch(index);
                    }
                }
                if ui
                    .add_enabled(
                        self.desktop.count() < Desktop::MAX_WORKSPACES,
                        egui::Button::new("+"),
                    )
                    .on_hover_text("New desktop")
                    .clicked()
                {
                    self.desktop.add_workspace();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(if guest_session() {
                        "AERA OS · Preview"
                    } else {
                        "Hosted session"
                    });
                });
            });
        });
        egui::TopBottomPanel::bottom("dock").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                for panel in Panel::ALL {
                    let label = if self.desktop.state(panel) == WindowState::Minimized {
                        format!("{} · minimized", panel.title())
                    } else {
                        panel.title().to_owned()
                    };
                    if ui
                        .selectable_label(self.desktop.is_open(panel), label)
                        .clicked()
                    {
                        self.launch(panel);
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            design::wallpaper(ui, self.dark);
            ui.add_space(64.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("A E R A").size(48.0).strong());
                ui.label("One OS. Any machine.");
                if self.glass_enabled && self.glass_available {
                    ui.add_space(12.0);
                    glass::preview(ui, self.dark, self.glass_quality, self.reduced_motion);
                }
                ui.add_space(20.0);
                ui.horizontal_wrapped(|ui| {
                    for panel in Panel::ALL {
                        if ui
                            .add_sized([110.0, 64.0], egui::Button::new(panel.title()))
                            .clicked()
                        {
                            self.launch(panel);
                        }
                    }
                });
                ui.add_space(12.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query).hint_text("Search applications"),
                );
                let matches = search(&self.query);
                if (response.has_focus() || response.lost_focus())
                    && ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
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

fn guest_session() -> bool {
    cfg!(target_os = "linux") && std::env::var_os("AERA_GUEST_SESSION").is_some()
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 480.0])
            .with_fullscreen(guest_session()),
        ..Default::default()
    };
    eframe::run_native(
        "AERA Shell",
        options,
        Box::new(|cc| {
            let dark = std::env::var_os("AERA_LIGHT").is_none();
            design::apply(&cc.egui_ctx, dark);
            let available =
                cc.wgpu_render_state.is_some() && std::env::var_os("AERA_GLASS_OFF").is_none();
            if available {
                if let Some(state) = &cc.wgpu_render_state {
                    glass::install(state);
                }
            }
            Ok(Box::new(Shell {
                dark,
                glass_available: available,
                glass_enabled: available,
                ..Shell::default()
            }))
        }),
    )
}
