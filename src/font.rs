use std::fs::read;

use egui::{FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};

pub fn load() -> FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    let handle = SystemSource::new()
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .unwrap();

    let f = handle.load().unwrap();

    let buf: Vec<u8> = match handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => read(path).unwrap(),
    };

    fonts
        .font_data
        .insert(f.full_name(), FontData::from_owned(buf));

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push(f.full_name().to_owned());
    }

    fonts
}
