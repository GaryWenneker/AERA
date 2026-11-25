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
            Color32::from_rgb(115, 215, 196),
        )
    } else {
        (
            Color32::from_rgb(240, 243, 245),
            Color32::WHITE,
            Color32::from_rgb(221, 229, 233),
            Color32::from_rgb(32, 43, 50),
            Color32::from_rgb(19, 108, 96),
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
        Color32::from_rgb(43, 83, 91)
    } else {
        Color32::from_rgb(187, 222, 217)
    };
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    ctx.set_style(style);
}

pub fn wallpaper(ui: &egui::Ui, dark: bool) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    let (a, b) = if dark {
        ([13.0, 23.0, 36.0], [47.0, 99.0, 105.0])
    } else {
        ([219.0, 232.0, 241.0], [151.0, 194.0, 195.0])
    };
    let mut mesh = egui::Mesh::default();
    for y in 0..=20 {
        for x in 0..=20 {
            let u = x as f32 / 20.0;
            let v = y as f32 / 20.0;
            let wave = (u * 3.0 + v * 2.4).sin().mul_add(0.38, 0.5);
            let color = |i: usize| (a[i] + (b[i] - a[i]) * wave) as u8;
            mesh.colored_vertex(
                egui::pos2(rect.left() + u * rect.width(), rect.top() + v * rect.height()),
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

pub fn dock_icon(ui: &mut egui::Ui, panel: Panel, running: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(52.0, 60.0), egui::Sense::click());
    let tile = egui::Rect::from_center_size(
        rect.center() - egui::vec2(0.0, 4.0),
        egui::vec2(44.0, 44.0),
    );
    let p = ui.painter();
    let color = match panel {
        Panel::Files => Color32::from_rgb(64, 143, 210),
        Panel::Disks => Color32::from_rgb(91, 115, 135),
        Panel::Settings => Color32::from_rgb(110, 125, 146),
        Panel::About => Color32::from_rgb(52, 141, 134),
    };
    p.rect_filled(tile, 12.0, color);
    if response.hovered() {
        p.rect_filled(tile, 12.0, Color32::from_white_alpha(35));
    }
    let c = tile.center();
    let white = Color32::WHITE;
    let stroke = Stroke::new(2.0_f32, white);
    match panel {
        Panel::Files => {
            let folder = egui::Rect::from_center_size(c + egui::vec2(0.0, 2.0), egui::vec2(27.0, 17.0));
            p.rect_filled(folder, 3.0, white);
            p.rect_filled(
                egui::Rect::from_min_size(folder.min - egui::vec2(0.0, 4.0), egui::vec2(12.0, 7.0)),
                2.0,
                white,
            );
        }
        Panel::Disks => {
            p.rect_stroke(
                egui::Rect::from_center_size(c, egui::vec2(25.0, 26.0)),
                5.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            p.line_segment([c + egui::vec2(-10.0, 5.0), c + egui::vec2(10.0, 5.0)], stroke);
            p.circle_filled(c + egui::vec2(6.0, 9.0), 1.5, white);
        }
        Panel::Settings => {
            p.circle_stroke(c, 10.0, Stroke::new(3.0_f32, white));
            p.circle_filled(c, 3.0, white);
            for n in 0..8 {
                let angle = n as f32 * std::f32::consts::TAU / 8.0;
                let d = egui::vec2(angle.cos(), angle.sin());
                p.line_segment([c + d * 10.0, c + d * 14.0], stroke);
            }
        }
        Panel::About => {
            p.circle_stroke(c, 13.0, stroke);
            p.text(c, egui::Align2::CENTER_CENTER, "i", egui::FontId::proportional(24.0), white);
        }
    }
    if running {
        p.circle_filled(egui::pos2(c.x, rect.bottom() - 3.0), 2.0, ui.visuals().text_color());
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
