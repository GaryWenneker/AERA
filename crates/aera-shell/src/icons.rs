//! Shared resolution-independent system symbols.
use aera_shell::Panel;
use eframe::egui::{self, Color32, Stroke};

#[derive(Clone, Copy)]
pub enum Icon {
    Files,
    Disks,
    Settings,
    About,
    Terminal,
    Monitor,
    Library,
    Installer,
    Search,
    Notifications,
    Network,
    Appearance,
}
impl Icon {
    pub const ALL: [Self; 12] = [
        Self::Files,
        Self::Disks,
        Self::Settings,
        Self::About,
        Self::Terminal,
        Self::Monitor,
        Self::Library,
        Self::Installer,
        Self::Search,
        Self::Notifications,
        Self::Network,
        Self::Appearance,
    ];
    pub fn app(panel: Panel) -> Self {
        match panel {
            Panel::Files => Self::Files,
            Panel::Disks => Self::Disks,
            Panel::Settings => Self::Settings,
            Panel::About => Self::About,
            Panel::Terminal => Self::Terminal,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Disks => "Disks",
            Self::Settings => "Settings",
            Self::About => "About",
            Self::Terminal => "Terminal",
            Self::Monitor => "Monitor",
            Self::Library => "App Library",
            Self::Installer => "Installer",
            Self::Search => "Search",
            Self::Notifications => "Notifications",
            Self::Network => "Network",
            Self::Appearance => "Appearance",
        }
    }
    fn tint(self) -> Color32 {
        match self {
            Self::Files | Self::Search | Self::Network => Color32::from_rgb(67, 116, 205),
            Self::Settings | Self::Disks | Self::Terminal => Color32::from_rgb(92, 107, 136),
            Self::About | Self::Appearance | Self::Library => Color32::from_rgb(126, 103, 183),
            Self::Notifications | Self::Installer => Color32::from_rgb(190, 116, 65),
            Self::Monitor => Color32::from_rgb(53, 132, 156),
        }
    }
}
pub fn paint(ui: &egui::Ui, rect: egui::Rect, icon: Icon, tile: bool) {
    let p = ui.painter();
    let scale = rect.width().min(rect.height()) / 24.0;
    let c = rect.center();
    let pt = |x: f32, y: f32| c + egui::vec2(x - 12.0, y - 12.0) * scale;
    let box_at = |x, y, w, h| egui::Rect::from_min_max(pt(x, y), pt(x + w, y + h));
    let tint = icon.tint();
    let ink = if tile {
        Color32::WHITE
    } else {
        ui.visuals().text_color()
    };
    let stroke = Stroke::new(1.65 * scale, ink);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], stroke);
    };
    let outline = |x, y, w, h| {
        p.rect_stroke(
            box_at(x, y, w, h),
            (2.0_f32 * scale) as u8,
            stroke,
            egui::StrokeKind::Inside,
        );
    };
    if tile {
        p.rect_filled(rect, (5.5_f32 * scale) as u8, tint);
        p.rect_stroke(
            rect,
            (5.5_f32 * scale) as u8,
            Stroke::new(0.65 * scale, Color32::from_white_alpha(80)),
            egui::StrokeKind::Inside,
        );
    }
    match icon {
        Icon::Files => {
            p.rect_filled(box_at(5.0, 7.0, 7.0, 4.0), 1, ink);
            p.rect_filled(box_at(5.0, 9.0, 14.0, 9.0), 2, ink);
        }
        Icon::Disks => {
            outline(5.0, 5.0, 14.0, 14.0);
            line((5.0, 14.0), (19.0, 14.0));
            p.circle_filled(pt(16.0, 16.5), scale, ink);
        }
        Icon::Settings => {
            p.circle_stroke(c, 5.0 * scale, stroke);
            p.circle_stroke(c, 2.0 * scale, stroke);
            for n in 0..8 {
                let a = n as f32 * std::f32::consts::TAU / 8.0;
                let d = egui::vec2(a.cos(), a.sin()) * scale;
                p.line_segment([c + d * 5.5, c + d * 7.5], stroke);
            }
        }
        Icon::About => {
            p.circle_stroke(c, 7.0 * scale, stroke);
            p.circle_filled(pt(12.0, 8.0), scale, ink);
            line((12.0, 11.0), (12.0, 16.0));
        }
        Icon::Terminal => {
            outline(4.0, 5.0, 16.0, 14.0);
            line((7.0, 9.0), (10.0, 12.0));
            line((10.0, 12.0), (7.0, 15.0));
            line((12.0, 15.0), (16.0, 15.0));
        }
        Icon::Monitor => {
            outline(4.0, 5.0, 16.0, 12.0);
            line((6.0, 12.0), (9.0, 12.0));
            line((9.0, 12.0), (11.0, 8.0));
            line((11.0, 8.0), (14.0, 14.0));
            line((14.0, 14.0), (18.0, 10.0));
            line((9.0, 19.0), (15.0, 19.0));
        }
        Icon::Library => {
            for y in [5.0, 13.0] {
                for x in [5.0, 13.0] {
                    p.rect_filled(box_at(x, y, 6.0, 6.0), 1, ink);
                }
            }
        }
        Icon::Installer => {
            outline(5.0, 15.0, 14.0, 5.0);
            line((12.0, 4.0), (12.0, 13.0));
            line((8.0, 9.0), (12.0, 13.0));
            line((16.0, 9.0), (12.0, 13.0));
        }
        Icon::Search => {
            p.circle_stroke(pt(10.5, 10.5), 5.0 * scale, stroke);
            line((14.0, 14.0), (19.0, 19.0));
        }
        Icon::Notifications => {
            p.circle_stroke(pt(12.0, 10.0), 4.5 * scale, stroke);
            line((7.5, 10.0), (6.0, 16.0));
            line((16.5, 10.0), (18.0, 16.0));
            line((6.0, 16.0), (18.0, 16.0));
            p.circle_filled(pt(12.0, 19.0), scale, ink);
        }
        Icon::Network => {
            for r in [4.0, 7.0, 10.0] {
                let points = (0..=12)
                    .map(|i| {
                        let a = -2.35 + i as f32 * 1.56 / 12.0;
                        pt(12.0, 18.0) + egui::vec2(a.cos(), a.sin()) * r * scale
                    })
                    .collect();
                p.add(egui::Shape::line(points, stroke));
            }
            p.circle_filled(pt(12.0, 18.0), scale, ink);
        }
        Icon::Appearance => {
            outline(5.0, 5.0, 10.0, 10.0);
            outline(9.0, 9.0, 10.0, 10.0);
        }
    }
}
pub fn row(ui: &mut egui::Ui, icon: Icon, label: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(17.0, 17.0), egui::Sense::hover());
        paint(ui, rect, icon, false);
        ui.button(label)
    })
    .inner
}
