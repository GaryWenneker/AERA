#![forbid(unsafe_code)]

use aera_shell::{search, Desktop, Panel, WindowState};
use eframe::egui::{self, Color32};
mod design;
mod disks;
mod files;
mod glass;
mod icons;
mod pty_terminal;
mod terminal;
mod wallpaper;
use icons::Icon;

use std::path::PathBuf;

struct Shell {
    desktop: Desktop,
    focus_request: Option<Panel>,
    terminal: terminal::Terminal,
    pty: Option<pty_terminal::Session>,
    pty_error: Option<String>,
    settings_page: usize,
    window_opacity: u8,
    query: String,
    launcher_open: bool,
    dark: bool,
    accent_index: usize,
    glass_enabled: bool,
    glass_available: bool,
    glass_quality: f32,
    reduced_motion: bool,
    zoom: f32,
    compact_dock: bool,
    wallpaper_texture: Option<egui::TextureHandle>,
    wallpaper_choice: usize,
    wallpaper_thumbs: Vec<egui::TextureHandle>,
    icon_view: bool,
    wallpaper_path: String,
    wallpaper_cover: bool,
    wallpaper_error: Option<String>,
    directory: Option<PathBuf>,
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
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
            focus_request: None,
            terminal: terminal::Terminal::default(),
            pty: None,
            pty_error: None,
            settings_page: 0,
            window_opacity: 82,
            query: String::new(),
            launcher_open: false,
            dark: false,
            accent_index: 0,
            glass_enabled: true,
            glass_available: false,
            glass_quality: 0.65,
            reduced_motion: false,
            zoom: 1.0,
            compact_dock: true,
            wallpaper_texture: None,
            wallpaper_choice: wallpaper::default_index(),
            wallpaper_thumbs: Vec::new(),
            icon_view: true,
            wallpaper_path: String::new(),
            wallpaper_cover: true,
            wallpaper_error: None,
            directory: home().or_else(|| std::env::current_dir().ok()),
            back: Vec::new(),
            forward: Vec::new(),
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
        let was_open = self.desktop.is_open(panel);
        self.desktop.set_open(panel, true);
        self.focus_request = Some(panel);
        if was_open {
            return;
        }
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

    fn load_directory(&mut self, path: PathBuf) -> bool {
        match files::list(&path) {
            Ok(entries) => {
                self.directory = Some(path);
                self.entries = entries;
                self.selected = None;
                self.file_error = None;
                true
            }
            Err(error) => {
                self.file_error = Some(error.to_string());
                false
            }
        }
    }

    fn navigate(&mut self, path: PathBuf) {
        let previous = self.directory.clone();
        if previous.as_ref() != Some(&path) && self.load_directory(path) {
            if let Some(previous) = previous {
                self.back.push(previous);
                if self.back.len() > 100 {
                    self.back.remove(0);
                }
            }
            self.forward.clear();
        }
    }

    fn navigate_history(&mut self, forward: bool) {
        let next = if forward {
            self.forward.last()
        } else {
            self.back.last()
        }
        .cloned();
        let previous = self.directory.clone();
        if let Some(next) = next {
            if self.load_directory(next) {
                let (source, dest) = if forward {
                    (&mut self.forward, &mut self.back)
                } else {
                    (&mut self.back, &mut self.forward)
                };
                source.pop();
                if let Some(previous) = previous {
                    dest.push(previous);
                }
            }
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
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(168.0);
                ui.strong("Favorites");
                for name in ["Home", "Desktop", "Documents", "Downloads", "Pictures"] {
                    if let Some(base) = home() {
                        let path = if name == "Home" {
                            base
                        } else {
                            base.join(name)
                        };
                        if path.is_dir()
                            && ui
                                .selectable_label(self.directory.as_ref() == Some(&path), name)
                                .clicked()
                        {
                            self.navigate(path);
                        }
                    }
                }
                ui.separator();
                ui.strong("Locations");
                if icons::row(ui, Icon::Disks, "Volumes").clicked() {
                    self.launch(Panel::Disks);
                }
            });
            ui.separator();
            ui.vertical(|ui| self.files_content(ui));
        });
    }

    fn files_content(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.back.is_empty(), egui::Button::new("←"))
                .on_hover_text("Back")
                .clicked()
            {
                self.navigate_history(false);
            }
            if ui
                .add_enabled(!self.forward.is_empty(), egui::Button::new("→"))
                .on_hover_text("Forward")
                .clicked()
            {
                self.navigate_history(true);
            }
            if ui.button("Home").clicked() {
                if let Some(path) = home() {
                    self.navigate(path);
                }
            }
            if ui.button("↑").on_hover_text("Parent folder").clicked() {
                if let Some(path) = self.directory.as_deref().and_then(std::path::Path::parent) {
                    self.navigate(path.to_path_buf());
                }
            }
            ui.selectable_value(&mut self.icon_view, true, "Icons");
            ui.selectable_value(&mut self.icon_view, false, "List");
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
            ui.add(
                egui::TextEdit::singleline(&mut self.file_filter)
                    .hint_text("Search this folder")
                    .desired_width(f32::INFINITY),
            );
        });
        let mut next = None;
        let filter = self.file_filter.to_lowercase();
        egui::ScrollArea::vertical()
            .max_height((ui.available_height() - 110.0).max(90.0))
            .show(ui, |ui| {
                if self.icon_view {
                    ui.horizontal_wrapped(|ui| {
                        for entry in &self.entries {
                            let name = entry.name.to_string_lossy();
                            if !name.to_lowercase().contains(&filter) {
                                continue;
                            }
                            let (rect, response) = ui
                                .allocate_exact_size(egui::vec2(94.0, 82.0), egui::Sense::click());
                            if self.selected.as_ref() == Some(&entry.path) {
                                ui.painter()
                                    .rect_filled(rect, 8, ui.visuals().selection.bg_fill);
                            }
                            let icon = egui::Rect::from_center_size(
                                rect.center_top() + egui::vec2(0.0, 28.0),
                                egui::vec2(43.0, 43.0),
                            );
                            design::file_icon(ui, icon, entry.kind, &entry.path);
                            let short: String = name.chars().take(14).collect();
                            ui.painter().text(
                                rect.center_bottom() - egui::vec2(0.0, 12.0),
                                egui::Align2::CENTER_CENTER,
                                short,
                                egui::FontId::proportional(12.0),
                                ui.visuals().text_color(),
                            );
                            if response.clicked() {
                                self.selected = Some(entry.path.clone());
                            }
                            if response.double_clicked() && entry.kind == "Folder" {
                                next = Some(entry.path.clone());
                            }
                            response.on_hover_text(name.as_ref());
                        }
                    });
                } else {
                    let width = ui.available_width().max(100.0);
                    let name_width = width * 0.60;
                    ui.horizontal(|ui| {
                        ui.add_sized([name_width, 22.0], egui::Label::new("Name"));
                        ui.label("Kind");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label("Size");
                        });
                    });
                    ui.separator();
                    for entry in &self.entries {
                        let name = entry.name.to_string_lossy();
                        if !name.to_lowercase().contains(&filter) {
                            continue;
                        }
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::click());
                        let selected = self.selected.as_ref() == Some(&entry.path);
                        if selected || response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                6,
                                if selected {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.visuals().faint_bg_color
                                },
                            );
                        }
                        let icon = egui::Rect::from_center_size(
                            rect.left_center() + egui::vec2(16.0, 0.0),
                            egui::vec2(21.0, 21.0),
                        );
                        design::file_icon(ui, icon, entry.kind, &entry.path);
                        let p = ui.painter();
                        let ink = ui.visuals().text_color();
                        let font = egui::FontId::proportional(13.0);
                        let clip = egui::Rect::from_min_max(
                            rect.min + egui::vec2(34.0, 0.0),
                            egui::pos2(rect.left() + name_width, rect.bottom()),
                        );
                        p.with_clip_rect(clip).text(
                            rect.left_center() + egui::vec2(34.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            name.as_ref(),
                            font.clone(),
                            ink,
                        );
                        p.text(
                            rect.left_center() + egui::vec2(name_width + 5.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            entry.kind,
                            font.clone(),
                            ink,
                        );
                        p.text(
                            rect.right_center() - egui::vec2(6.0, 0.0),
                            egui::Align2::RIGHT_CENTER,
                            files::display_size(entry.size),
                            font,
                            ink,
                        );
                        if response.clicked() {
                            self.selected = Some(entry.path.clone());
                        }
                        if response.double_clicked() && entry.kind == "Folder" {
                            next = Some(entry.path.clone());
                        }
                        response.on_hover_text(name.as_ref());
                    }
                }
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
        ui.collapsing("File actions", |ui| {
            ui.small("Create or copy in this folder without overwrite");
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
                let copyable = self.entries.iter().any(|entry| {
                    entry.kind == "File" && self.selected.as_ref() == Some(&entry.path)
                });
                if ui
                    .add_enabled(ready && copyable, egui::Button::new("Copy selected"))
                    .clicked()
                {
                    self.file_action(2);
                }
            });
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
            Panel::Terminal => {
                ui.horizontal(|ui| {
                    ui.strong("Shell");
                    ui.small(if guest_session() {
                        "Guest"
                    } else {
                        "Host · local user permissions"
                    });
                    if ui.button("Restart session").clicked() {
                        self.pty = None;
                        self.pty_error = None;
                    }
                });
                if self.pty.is_none() && self.pty_error.is_none() {
                    match pty_terminal::Session::start(ctx) {
                        Ok(session) => self.pty = Some(session),
                        Err(error) => self.pty_error = Some(error),
                    }
                }
                if let Some(session) = &mut self.pty {
                    session.show(ui);
                } else if let Some(error) = &self.pty_error {
                    ui.label(format!("PTY unavailable: {error}"));
                    ui.small("Fallback command runner");
                    self.terminal.show(ui, guest_session());
                }
            }
            Panel::Files => self.files_view(ui),
            Panel::Disks => self.disks_view(ui),
            Panel::Settings => {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(145.0);
                        ui.heading("Settings");
                        for (index, (icon, label)) in [
                            (Icon::Appearance, "Appearance"),
                            (Icon::Files, "Wallpaper"),
                            (Icon::Settings, "Glass & motion"),
                            (Icon::Disks, "System"),
                        ].into_iter().enumerate() {
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                                icons::paint(ui, rect, icon, true);
                                if ui.selectable_label(self.settings_page == index, label).clicked() {
                                    self.settings_page = index;
                                }
                            });
                        }
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        match self.settings_page {
                    0 => design::section(ui, "Appearance", |ui| {
                if ui.checkbox(&mut self.dark, "Dark appearance").changed() {
                    design::apply(ctx, self.dark, self.accent_index);
                }
                if ui
                    .add(egui::Slider::new(&mut self.zoom, 0.8..=1.6).text("UI zoom"))
                    .changed()
                {
                    ctx.set_zoom_factor(self.zoom);
                }
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("Accent");
                    for (index, name) in design::ACCENTS.iter().enumerate() {
                        if ui.selectable_value(&mut self.accent_index, index, *name).changed() {
                            design::apply(ctx, self.dark, self.accent_index);
                        }
                    }
                });
                ui.checkbox(&mut self.compact_dock, "Compact dock");
                ui.add(
                    egui::Slider::new(&mut self.window_opacity, 65..=100)
                        .suffix("%")
                        .text("Window opacity"),
                );
                }),
                    1 => design::section(ui, "Wallpaper", |ui| {
                    if self.wallpaper_thumbs.is_empty() {
                        for index in 0..wallpaper::NAMES.len() {
                            self.wallpaper_thumbs.push(wallpaper::upload(
                                ctx,
                                format!("wallpaper-thumb-{index}"),
                                wallpaper::thumbnail(index),
                            ));
                        }
                    }
                    let mut chosen = None;
                    ui.horizontal_wrapped(|ui| {
                        for (index, name) in wallpaper::NAMES.iter().enumerate() {
                            if !wallpaper::available(index) {
                                continue;
                            }
                            ui.vertical(|ui| {
                                let image = egui::Image::new(&self.wallpaper_thumbs[index])
                                    .fit_to_exact_size(egui::vec2(112.0, 63.0))
                                    .corner_radius(8)
                                    .sense(egui::Sense::click());
                                if ui.add(image).clicked()
                                    || ui.selectable_label(self.wallpaper_choice == index, *name).clicked()
                                {
                                    chosen = Some(index);
                                }
                            });
                        }
                    });
                    if let Some(index) = chosen {
                        self.wallpaper_texture = Some(wallpaper::upload(
                            ctx,
                            "desktop-wallpaper",
                            wallpaper::builtin(index),
                        ));
                        self.wallpaper_choice = index;
                        self.wallpaper_error = None;
                    }
                    if !wallpaper::available(0) {
                        ui.small("Photo pack not installed. Run os/fetch-wallpapers.py and rebuild.");
                    }
                    ui.label("Local image (PNG, JPEG, WebP)");
                    ui.add(egui::TextEdit::singleline(&mut self.wallpaper_path).hint_text("Image file path").desired_width(f32::INFINITY));
                    if ui.button("Apply image").clicked() {
                        match wallpaper::load(std::path::Path::new(&self.wallpaper_path)) {
                            Ok(image) => {
                                self.wallpaper_texture =
                                    Some(wallpaper::upload(ctx, "desktop-wallpaper", image));
                                self.wallpaper_choice = wallpaper::NAMES.len();
                                self.wallpaper_error = None;
                            }
                            Err(error) => self.wallpaper_error = Some(error),
                        }
                    }
                    ui.checkbox(&mut self.wallpaper_cover, "Fill screen (crop)");
                    if let Some(error) = &self.wallpaper_error {
                        ui.colored_label(Color32::LIGHT_RED, error);
                    }
                }),
                    2 => design::section(ui, "Glass & motion", |ui| {
                ui.add_enabled(
                    self.glass_available,
                    egui::Checkbox::new(&mut self.glass_enabled, "Enable GPU glass"),
                );
                ui.add(egui::Slider::new(&mut self.glass_quality, 0.0..=1.0).text("Blur quality"));
                ui.checkbox(&mut self.reduced_motion, "Reduce motion");
                ui.collapsing("Advanced material preview", |ui| {
                    if self.glass_enabled && self.glass_available {
                        glass::preview(ui, self.dark, self.glass_quality, self.reduced_motion);
                    }
                });
                }),
                    _ => design::section(ui, "Storage & system", |ui| {
                if icons::row(ui, Icon::Disks, "Storage").clicked() {
                    self.launch(Panel::Disks);
                }
                if icons::row(ui, Icon::About, "System information").clicked() {
                    self.launch(Panel::About);
                }
                ui.collapsing("System icon library", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for icon in Icon::ALL {
                            ui.vertical(|ui| {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
                                icons::paint(ui, rect, icon, true);
                                ui.small(icon.label());
                            });
                        }
                    });
                });
                }),
                        }
                        ui.small("Session settings");
                    });
                });
            }
            Panel::About => {
                let target = aera_core::Target::current();
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("AERA").size(32.0).strong());
                    ui.label("One OS. Any machine.");
                    ui.add_space(12.0);
                });
                ui.separator();
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
            if design::traffic(ui, Color32::from_rgb(237, 106, 101), "×", "Close") {
                self.desktop.set_state(panel, WindowState::Closed);
            }
            if design::traffic(ui, Color32::from_rgb(230, 183, 78), "−", "Minimize") {
                self.desktop.set_state(panel, WindowState::Minimized);
            }
            let maximized = self.desktop.state(panel) == WindowState::Maximized;
            if design::traffic(
                ui,
                Color32::from_rgb(92, 190, 124),
                "↗",
                if maximized { "Restore" } else { "Maximize" },
            ) {
                self.desktop.set_state(
                    panel,
                    if maximized {
                        WindowState::Floating
                    } else {
                        WindowState::Maximized
                    },
                );
            }
            ui.add_space(12.0);
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
        let alpha = if matches!(panel, Panel::Terminal | Panel::Settings | Panel::Files) {
            255
        } else if self.glass_enabled {
            (u16::from(self.window_opacity) * 255 / 100) as u8
        } else {
            255
        };
        let fill = if self.dark {
            Color32::from_rgba_unmultiplied(28, 32, 44, alpha)
        } else {
            Color32::from_rgba_unmultiplied(250, 251, 255, alpha)
        };
        let frame = egui::Frame::window(&ctx.style()).fill(fill);
        egui::Window::new(panel.title())
            .id(egui::Id::new(("app", workspace, panel as usize)))
            .default_pos([80.0 + offset, 80.0 + offset])
            .default_size(
                if matches!(
                    panel,
                    Panel::Files | Panel::Disks | Panel::Terminal | Panel::Settings
                ) {
                    [650.0, 380.0]
                } else {
                    [380.0, 260.0]
                },
            )
            .resizable([true, true])
            .min_size([240.0, 140.0])
            .frame(frame)
            .title_bar(false)
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                self.window_controls(ui, panel);
                if panel == Panel::Terminal {
                    self.app_content(ctx, ui, panel);
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.app_content(ctx, ui, panel));
                }
            });
    }
}

impl eframe::App for Shell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.wallpaper_texture.is_none() {
            self.wallpaper_texture = Some(wallpaper::upload(
                ctx,
                "desktop-wallpaper",
                wallpaper::builtin(self.wallpaper_choice),
            ));
        }
        self.terminal.poll(ctx);
        if let Some(session) = &mut self.pty {
            session.poll();
        }
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
        let translucent = self.glass_enabled && self.glass_available;
        let chrome = design::chrome(self.dark, translucent);
        egui::TopBottomPanel::top("status")
            .frame(
                egui::Frame::new()
                    .fill(chrome)
                    .inner_margin(egui::Margin::symmetric(12, 1)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("AERA", |ui| {
                        for panel in Panel::ALL {
                            if icons::row(ui, Icon::app(panel), panel.title()).clicked() {
                                self.launch(panel);
                                ui.close_menu();
                            }
                        }
                    });
                    ui.menu_button("Apps", |ui| {
                        for panel in Panel::ALL {
                            if ui.button(panel.title()).clicked() {
                                self.launch(panel);
                                ui.close_menu();
                            }
                        }
                    });
                    if ui.button("Search").clicked() {
                        self.launcher_open = !self.launcher_open;
                    }
                    ui.separator();
                    for index in 0..self.desktop.count() {
                        if ui
                            .selectable_label(
                                self.desktop.active() == index,
                                format!("{}", index + 1),
                            )
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
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(ctx.style().visuals.panel_fill))
            .show(ctx, |ui| {
                if let Some(texture) = &self.wallpaper_texture {
                    wallpaper::paint(ui, texture, self.wallpaper_cover);
                } else {
                    design::wallpaper(ui, self.dark);
                }
            });
        egui::Area::new(egui::Id::new("floating-dock"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -12.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(chrome)
                    .corner_radius(egui::CornerRadius::same(24))
                    .shadow(egui::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 1,
                        color: Color32::from_black_alpha(45),
                    })
                    .stroke(egui::Stroke::new(1.0_f32, Color32::from_white_alpha(70)))
                    .inner_margin(egui::Margin::symmetric(12, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            for panel in Panel::ALL {
                                if design::dock_icon(
                                    ui,
                                    panel,
                                    self.desktop.is_open(panel),
                                    if self.compact_dock { 32.0 } else { 40.0 },
                                    self.reduced_motion,
                                )
                                .clicked()
                                {
                                    self.launch(panel);
                                }
                            }
                        });
                    });
            });
        if self.launcher_open {
            let mut open = self.launcher_open;
            egui::Window::new("Search applications")
                .open(&mut open)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 64.0))
                .resizable(false)
                .show(ctx, |ui| {
                    let response = ui.text_edit_singleline(&mut self.query);
                    let matches = search(&self.query);
                    if (response.has_focus() || response.lost_focus())
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        if let Some(panel) = matches.first() {
                            self.launch(*panel);
                            self.launcher_open = false;
                        }
                    }
                    for panel in matches {
                        if ui.button(panel.title()).clicked() {
                            self.launch(panel);
                            self.launcher_open = false;
                        }
                    }
                });
            self.launcher_open &= open;
        }
        for panel in Panel::ALL {
            self.panel(ctx, panel);
        }
        if let Some(panel) = self.focus_request.take() {
            ctx.move_to_top(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::new(("app", self.desktop.active(), panel as usize)),
            ));
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
            let dark =
                std::env::var_os("AERA_DARK").is_some() && std::env::var_os("AERA_LIGHT").is_none();
            design::apply(&cc.egui_ctx, dark, 0);
            let safe_mode = std::env::var_os("AERA_SAFE_MODE").is_some();
            let available = cc.wgpu_render_state.is_some()
                && std::env::var_os("AERA_GLASS_OFF").is_none()
                && !safe_mode;
            if available {
                if let Some(state) = &cc.wgpu_render_state {
                    glass::install(state);
                }
            }
            Ok(Box::new(Shell {
                dark,
                    egui::TextureOptions::LINEAR,
                )),
                glass_available: available,
                glass_enabled: available,
                ..Shell::default()
            }))
        }),
    )
}
