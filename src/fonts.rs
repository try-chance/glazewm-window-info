use std::fs;
use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

pub(crate) fn configure_fonts(ctx: &egui::Context) {
    let candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];

    let Some(font_bytes) = candidates.iter().find_map(|path| fs::read(path).ok()) else {
        return;
    };

    let font_name = "system-cjk".to_owned();
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        font_name.clone(),
        Arc::new(FontData::from_owned(font_bytes)),
    );

    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        family.insert(0, font_name.clone());
    }
    if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
        family.push(font_name);
    }

    ctx.set_fonts(fonts);
}
