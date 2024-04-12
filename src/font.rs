use egui::{FontDefinitions, FontFamily};

pub fn load() -> FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "berkeley".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "/home/rumpl/.local/share/fonts/BerkeleyMono-Regular.ttf"
        )),
    );

    fonts.font_data.insert(
        "berkeley-bold".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "/home/rumpl/.local/share/fonts/BerkeleyMono-Bold.ttf"
        )),
    );

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "berkeley".to_owned());

    fonts.families.insert(
        FontFamily::Name("berkeley".into()),
        vec!["berkeley".to_owned()],
    );

    fonts.families.insert(
        FontFamily::Name("berkeley-bold".into()),
        vec!["berkeley-bold".to_owned()],
    );

    fonts
}
