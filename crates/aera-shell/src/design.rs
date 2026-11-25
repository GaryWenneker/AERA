use crate::icons::{self, Icon};
use aera_shell::Panel;
use eframe::egui::{self, Color32, Stroke};

pub fn apply(ctx: &egui::Context, dark: bool) {
    let mut style = (*ctx.style()).clone();
    style.visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let (canvas, surface, elevated, text, accent) = if dark {
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
    style.visuals.panel_fill = canvas;
    style.visuals.window_fill = surface;
    style.visuals.override_text_color = Some(text);
    style.visuals.widgets.inactive.bg_fill = elevated;
    style.visuals.widgets.inactive.weak_bg_fill = elevated;
    style.visuals.selection.stroke = Stroke::new(1.0_f32, accent);
    style.visuals.window_corner_radius = egui::CornerRadius::same(12);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(10);
    style.visuals.selection.bg_fill = if dark {
        Color32::from_rgb(51, 69, 111)
    } else {
        Color32::from_rgb(211, 222, 248)
    };
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
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
        (true, true) => Color32::from_rgba_unmultiplied(27, 36, 47, 218),
        (false, true) => Color32::from_rgba_unmultiplied(245, 249, 251, 230),
        (true, false) => Color32::from_rgb(27, 36, 47),
        (false, false) => Color32::from_rgb(245, 249, 251),
    }
}

pub fn dock_icon(ui: &mut egui::Ui, panel: Panel, running: bool, size: f32) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(size + 8.0, size + 12.0), egui::Sense::click());
    let tile =
        egui::Rect::from_center_size(rect.center() - egui::vec2(0.0, 4.0), egui::vec2(size, size));
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
