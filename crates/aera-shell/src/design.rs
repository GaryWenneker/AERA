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
    style.visuals.selection.stroke = Stroke::new(1.0, accent);
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    ctx.set_style(style);
}

pub fn wallpaper(ui: &egui::Ui, dark: bool) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    let color = if dark {
        Color32::from_rgb(23, 35, 41)
    } else {
        Color32::from_rgb(226, 235, 237)
    };
    let center = egui::pos2(rect.right() - rect.width() * 0.18, rect.bottom());
    for index in 1..=7 {
        painter.circle_stroke(
            center,
            rect.width() * index as f32 / 14.0,
            Stroke::new(2.0, color),
        );
    }
}
