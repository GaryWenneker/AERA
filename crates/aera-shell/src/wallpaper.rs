//! Local desktop artwork and aspect-preserving texture placement.
use eframe::egui::{self, Color32};
use std::path::Path;

pub const NAMES: [&str; 3] = ["Alpine dawn", "Coastal dusk", "Silk"];
const W: usize = 1280;
const H: usize = 720;

pub fn builtin(index: usize) -> egui::ColorImage {
    let palettes = [
        [[157, 184, 230], [255, 217, 185], [134, 151, 183], [90, 110, 150], [48, 68, 105]],
        [[58, 62, 112], [245, 166, 150], [119, 105, 151], [72, 82, 132], [35, 48, 83]],
        [[218, 230, 250], [245, 209, 228], [181, 176, 224], [131, 150, 209], [91, 104, 165]],
    ];
    let p = palettes[index.min(2)];
    let mix = |a: [u8; 3], b: [u8; 3], t: f32| {
        let c = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t) as u8;
        Color32::from_rgb(c(0), c(1), c(2))
    };
    let mut pixels = Vec::with_capacity(W * H);
    for y in 0..H {
        let v = y as f32 / H as f32;
        for x in 0..W {
            let u = x as f32 / W as f32;
            let mut color = mix(p[0], p[1], v);
            if index == 2 {
                let wave = (u * 7.0 + v * 5.0 + (v * 4.0).sin()).sin() * 0.5 + 0.5;
                color = mix(p[0], p[2], wave);
                let glow = ((u - 0.8).powi(2) + (v - 0.2).powi(2)).sqrt();
                color = mix(color.to_array()[..3].try_into().unwrap_or(p[0]), p[1], (1.0 - glow).clamp(0.0, 1.0) * 0.7);
            } else {
                for layer in 0..3 {
                    let l = layer as f32;
                    let ridge = 0.48 + l * 0.15
                        + (u * (9.0 + l * 2.0) + l * 1.8).sin() * 0.065
                        + (u * 23.0 + l).sin() * 0.023
                        + (u * 43.0 + l * 3.0).sin() * 0.009;
                    if v > ridge {
                        color = mix(p[layer + 2], p[layer + 2], 0.0);
                    }
                }
            }
            pixels.push(color);
        }
    }
    egui::ColorImage { size: [W, H], pixels }
}

pub fn load(path: &Path) -> Result<egui::ColorImage, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
        return Err("Image exceeds 64 MiB".into());
    }
    let mut reader = image::ImageReader::new(std::io::BufReader::new(file))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|e| e.to_string())?;
    let rgba = decoded.thumbnail(3840, 2160).to_rgba8();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}

pub fn placement(rect: egui::Rect, aspect: f32, cover: bool) -> (egui::Rect, egui::Rect) {
    let view = rect.width() / rect.height().max(1.0);
    let full = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
    if cover {
        let size = egui::vec2((view / aspect).min(1.0), (aspect / view).min(1.0));
        (rect, egui::Rect::from_center_size(egui::pos2(0.5, 0.5), size))
    } else {
        let size = egui::vec2(rect.width().min(rect.height() * aspect), rect.height().min(rect.width() / aspect));
        (egui::Rect::from_center_size(rect.center(), size), full)
    }
}

pub fn paint(ui: &egui::Ui, texture: &egui::TextureHandle, cover: bool) {
    let rect = ui.max_rect();
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let size = texture.size_vec2();
    let (dest, uv) = placement(rect, size.x / size.y, cover);
    ui.painter_at(rect).image(texture.id(), dest, uv, Color32::WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wide_image_cover_crops_horizontally() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0));
        let (dest, uv) = placement(rect, 2.0, true);
        assert_eq!(dest, rect);
        assert_eq!(uv.width(), 0.5);
        assert_eq!(uv.height(), 1.0);
        let (dest, _) = placement(rect, 2.0, false);
        assert_eq!(dest.height(), 50.0);
    }
}
