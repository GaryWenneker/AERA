#![forbid(unsafe_code)]

use aera_shell::{search, Desktop, Panel, WindowState};
use eframe::egui::{self, Color32};
mod design;
mod disks;
mod files;
mod glass;
mod icons;
mod terminal;
mod pty_terminal;
mod wallpaper;
use icons::Icon;

use std::path::PathBuf;

struct Shell {
    desktop: Desktop,
    terminal: terminal::Terminal,
    pty: Option<pty_terminal::Session>,
    pty_error: Option<String>,
    settings_page: usize,
    window_opacity: u8,
    query: String,
    launcher_open: bool,
    dark: bool,
    glass_enabled: bool,
    glass_available: bool,
    glass_quality: f32,
    reduced_motion: bool,
    zoom: f32,
    compact_dock: bool,
    wallpaper_texture: Option<egui::TextureHandle>,
    wallpaper_choice: usize,
    wallpaper_path: String,
    wallpaper_cover: bool,
    wallpaper_error: Option<String>,
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
            terminal: terminal::Terminal::default(),
            pty: None,
            pty_error: None,
            settings_page: 0,
            window_opacity: 82,
            query: String::new(),
            launcher_open: false,
            dark: false,
            glass_enabled: true,
            glass_available: false,
            glass_quality: 0.65,
            reduced_motion: false,
            zoom: 1.0,
            compact_dock: true,
            wallpaper_texture: None,
            wallpaper_choice: 0,
            wallpaper_path: String::new(),
            wallpaper_cover: true,
            wallpaper_error: None,
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
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(126.0);
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
            Panel::Terminal => {
                ui.horizontal(|ui| {
                    ui.strong("Shell");
                    ui.small(if guest_session() { "Guest" } else { "Host · local user permissions" });
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
                    design::apply(ctx, self.dark);
                }
                if ui
                    .add(egui::Slider::new(&mut self.zoom, 0.8..=1.6).text("UI zoom"))
                    .changed()
                {
                    ctx.set_zoom_factor(self.zoom);
                }
                ui.separator();
                ui.checkbox(&mut self.compact_dock, "Compact dock");
                ui.add(
                    egui::Slider::new(&mut self.window_opacity, 65..=100)
                        .suffix("%")
                        .text("Window opacity"),
                );
                }),
                    1 => design::section(ui, "Wallpaper", |ui| {
                    for (index, name) in wallpaper::NAMES.iter().enumerate() {
                        if ui
                            .selectable_label(self.wallpaper_choice == index, *name)
                            .clicked()
                        {
                            self.wallpaper_texture = Some(ctx.load_texture(
                                "desktop-wallpaper",
                                wallpaper::builtin(index),
                                egui::TextureOptions::LINEAR,
                            ));
                            self.wallpaper_choice = index;
                            self.wallpaper_error = None;
                        }
                    }
                    ui.label("Local image (PNG, JPEG, WebP)");
                    ui.text_edit_singleline(&mut self.wallpaper_path);
                    if ui.button("Apply image").clicked() {
                        match wallpaper::load(std::path::Path::new(&self.wallpaper_path)) {
                            Ok(image) => {
                                self.wallpaper_texture = Some(ctx.load_texture(
                                    "desktop-wallpaper",
                                    image,
                                    egui::TextureOptions::LINEAR,
                                ));
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
        let alpha = if matches!(panel, Panel::Terminal | Panel::Settings) { 255 } else if self.glass_enabled {
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
                if matches!(panel, Panel::Files | Panel::Disks | Panel::Terminal) {
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
        self.terminal.poll(ctx);
        if let Some(session) = &mut self.pty { session.poll(); }
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
        egui::CentralPanel::default().frame(egui::Frame::new().fill(ctx.style().visuals.panel_fill)).show(ctx, |ui| {
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
                    .corner_radius(egui::CornerRadius::same(16))
                    .shadow(egui::Shadow { offset: [0, 4], blur: 16, spread: 1, color: Color32::from_black_alpha(45) })
                    .stroke(egui::Stroke::new(1.0_f32, Color32::from_white_alpha(70)))
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            for panel in Panel::ALL {
                                if design::dock_icon(
                                    ui,
                                    panel,
                                    self.desktop.is_open(panel),
                                    if self.compact_dock { 32.0 } else { 40.0 },
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
                wallpaper_texture: Some(cc.egui_ctx.load_texture(
                    "desktop-wallpaper",
                    wallpaper::builtin(0),
                    egui::TextureOptions::LINEAR,
                )),
                glass_available: available,
                glass_enabled: available,
                ..Shell::default()
            }))
        }),
    )
}
