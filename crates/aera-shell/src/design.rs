use crate::icons::{self, Icon};
use aera_shell::Panel;
use eframe::egui::{self, Color32, Stroke};

pub const ACCENTS: [&str; 4] = ["Ocean", "Violet", "Rose", "Graphite"];

pub fn apply(ctx: &egui::Context, dark: bool, accent_index: usize) {
    let mut style = (*ctx.style()).clone();
    style.visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let (canvas, surface, elevated, text, default_accent) = if dark {
        (
            Color32::from_rgb(16, 20, 25),
            Color32::from_rgb(27, 34, 42),
            Color32::from_rgb(39, 49, 59),
            Color32::from_rgb(232, 237, 242),
            Color32::from_rgb(124, 156, 240),
        )
    } else {
        (
            Color32::from_rgb(245, 246, 250),
            Color32::WHITE,
            Color32::from_rgb(221, 229, 233),
            Color32::from_rgb(32, 43, 50),
            Color32::from_rgb(58, 96, 187),
        )
    };
    let accent = match (accent_index, dark) {
        (1, true) => Color32::from_rgb(175, 155, 240),
        (1, false) => Color32::from_rgb(111, 82, 183),
        (2, true) => Color32::from_rgb(237, 148, 177),
        (2, false) => Color32::from_rgb(173, 65, 103),
        (3, true) => Color32::from_rgb(187, 195, 210),
        (3, false) => Color32::from_rgb(83, 91, 112),
        _ => default_accent,
    };
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
    style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(20.0));
    style.visuals.panel_fill = canvas;
    style.visuals.extreme_bg_color = if dark { Color32::from_rgb(20, 24, 34) } else { Color32::from_rgb(232, 236, 245) };
    style.visuals.faint_bg_color = if dark { Color32::from_rgb(35, 40, 52) } else { Color32::from_rgb(238, 241, 247) };
    style.visuals.window_fill = surface;
    style.visuals.override_text_color = Some(text);
    style.visuals.widgets.inactive.bg_fill = elevated;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.weak_bg_fill = if dark {
        Color32::from_white_alpha(24)
    } else {
        Color32::from_black_alpha(16)
    };
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.noninteractive,
    ] {
        widget.corner_radius = egui::CornerRadius::same(7);
    }
    style.visuals.selection.stroke = Stroke::new(1.0_f32, accent);
    style.visuals.window_corner_radius = egui::CornerRadius::same(16);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(10);
    let base: [u8; 3] = if dark { [30, 34, 44] } else { [247, 248, 252] };
    let rgb = accent.to_array();
    let mix = |i: usize| ((u16::from(base[i]) * 3 + u16::from(rgb[i])) / 4) as u8;
    style.visuals.selection.bg_fill = Color32::from_rgb(mix(0), mix(1), mix(2));
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size = egui::vec2(36.0, 26.0);
    ctx.set_style(style);
}

pub fn wallpaper(ui: &egui::Ui, dark: bool) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    let (a, b) = if dark {
        ([13.0, 23.0, 36.0], [68.0, 72.0, 121.0])
    } else {
        ([231.0, 237.0, 251.0], [177.0, 166.0, 225.0])
    };
    let mut mesh = egui::Mesh::default();
    for y in 0..=20 {
        for x in 0..=20 {
            let u = x as f32 / 20.0;
            let v = y as f32 / 20.0;
            let wave = (u * 3.0 + v * 2.4).sin().mul_add(0.38, 0.5);
            let warmth = if dark { 0.0 } else { (u * v).powf(2.0) * 0.65 };
            let peach = [242.0, 194.0, 178.0];
            let color = |i: usize| {
                ((a[i] + (b[i] - a[i]) * wave) * (1.0 - warmth) + peach[i] * warmth) as u8
            };
            mesh.colored_vertex(
                egui::pos2(
                    rect.left() + u * rect.width(),
                    rect.top() + v * rect.height(),
                ),
                Color32::from_rgb(color(0), color(1), color(2)),
            );
            if x < 20 && y < 20 {
                let i = y * 21 + x;
                mesh.add_triangle(i, i + 1, i + 21);
                mesh.add_triangle(i + 1, i + 22, i + 21);
            }
        }
    }
    painter.add(egui::Shape::mesh(mesh));
    let center = egui::pos2(rect.right(), rect.bottom());
    for index in 1..=8 {
        painter.circle_stroke(
            center,
            rect.width() * index as f32 / 10.0,
            Stroke::new(1.5_f32, Color32::from_white_alpha(16)),
        );
    }
}

pub fn chrome(dark: bool, translucent: bool) -> Color32 {
    match (dark, translucent) {
        (true, true) => Color32::from_rgba_unmultiplied(30, 35, 48, 185),
        (false, true) => Color32::from_rgba_unmultiplied(245, 249, 251, 185),
        (true, false) => Color32::from_rgb(27, 36, 47),
        (false, false) => Color32::from_rgb(245, 249, 251),
    }
}

pub fn dock_icon(ui: &mut egui::Ui, panel: Panel, running: bool, size: f32, reduced_motion: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(size + 16.0, size + 22.0), egui::Sense::click());
    let hover = if reduced_motion {
        0.0
    } else {
        ui.ctx().animate_bool_with_time(response.id, response.hovered(), 0.16)
    };
    let visual_size = size + hover * 12.0;
    let tile = egui::Rect::from_center_size(
        rect.center() - egui::vec2(0.0, 4.0 + hover * 4.0),
        egui::vec2(visual_size, visual_size),
    );
    icons::paint(ui, tile, Icon::app(panel), true);
    if response.hovered() {
        ui.painter()
            .rect_filled(tile, 8, Color32::from_white_alpha(28));
    }
    if running {
        ui.painter().circle_filled(
            egui::pos2(rect.center().x, rect.bottom() - 2.0),
            1.8,
            ui.visuals().text_color(),
        );
    }
    response.on_hover_text(panel.title())
}

pub fn traffic(ui: &mut egui::Ui, color: Color32, glyph: &str, hint: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(14.0, 18.0), egui::Sense::click());
    ui.painter().circle_filled(rect.center(), 6.0, color);
    if response.hovered() {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            egui::FontId::proportional(11.0),
            Color32::from_rgb(50, 40, 40),
        );
    }
    response.on_hover_text(hint).clicked()
}

pub fn section(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(title).size(15.0).strong());
    let dark = ui.visuals().dark_mode;
    let fill = if dark {
        Color32::from_rgb(34, 40, 52)
    } else {
        Color32::from_rgb(255, 255, 255)
    };
    let edge = if dark {
        Color32::from_white_alpha(20)
    } else {
        Color32::from_black_alpha(15)
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, edge))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            content(ui);
        });
}

pub fn file_icon(ui: &egui::Ui, rect: egui::Rect, kind: &str, path: &std::path::Path) {
    let painter = ui.painter();
    let size = rect.width().min(rect.height());
    let origin = rect.center() - egui::vec2(size, size) * 0.5;
    let point = |x: f32, y: f32| origin + egui::vec2(x, y) * size;
    let bounds = |x: f32, y: f32, w: f32, h: f32| {
        egui::Rect::from_min_max(point(x, y), point(x + w, y + h))
    };
    if kind == "Folder" {
        painter.rect_filled(bounds(0.08, 0.17, 0.39, 0.26), 3, Color32::from_rgb(64, 126, 205));
        painter.rect_filled(bounds(0.08, 0.29, 0.84, 0.58), 4, Color32::from_rgb(91, 158, 234));
        painter.line_segment(
            [point(0.14, 0.33), point(0.86, 0.33)],
            Stroke::new(1.0_f32, Color32::from_white_alpha(125)),
        );
    } else {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        let accent = match ext.as_str() {
            "png" | "jpg" | "jpeg" | "webp" => Color32::from_rgb(122, 100, 190),
            "rs" | "js" | "ts" | "py" | "toml" => Color32::from_rgb(65, 117, 190),
            "pdf" => Color32::from_rgb(193, 78, 83),
            _ => Color32::from_rgb(117, 129, 151),
        };
        painter.rect_filled(bounds(0.18, 0.06, 0.64, 0.86), 3, Color32::from_rgb(242, 245, 251));
        painter.rect_stroke(bounds(0.18, 0.06, 0.64, 0.86), 3, Stroke::new(0.8_f32, accent), egui::StrokeKind::Inside);
        for y in [0.37, 0.49] {
            painter.line_segment([point(0.3, y), point(0.68, y)], Stroke::new(1.0_f32, accent));
        }
        if size >= 32.0 {
            let label: String = ext.chars().take(4).collect::<String>().to_uppercase();
            painter.text(point(0.5, 0.73), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(9.0), accent);
        }
    }
}
