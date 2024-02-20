pub fn about_screen(ui: &mut egui::Ui) {
    ui.heading("About");
    ui.label("Perhabs provides simple exercises for the brain.");
    ui.label("The exercises it provides are all inspired by or downright copied from wonderful therapists all around the world. None of this is original, it is merely gathered here for your convenience.",);
    ui.separator();
    ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
}
