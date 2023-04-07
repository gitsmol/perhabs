use tts::Tts;

use crate::{
    asset_loader::AppData,
    windowman::{AppWin, View},
};

pub struct DebugInfo {}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {}
    }
}

impl DebugInfo {}

impl AppWin for DebugInfo {
    fn name(&self) -> &'static str {
        "Debugging info"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, appdata: &AppData, tts: &mut Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, appdata, tts));
    }
}

impl View for DebugInfo {
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _tts: &mut Tts) {
        egui::Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                if let Some(config) = &appdata.config {
                    ui.label("PerhabsConfig source:");
                    ui.label(&config.source.to_string());
                    ui.end_row();
                } else {
                    ui.label("No PerhabsConfig found. This is bad.");
                }
                if let Some(excconfig) = &appdata.excconfig {
                    ui.label("ExcConfig source:");
                    ui.label(&excconfig.source.to_string());
                    ui.end_row();
                } else {
                    ui.label("No ExcConfig found. This is bad.");
                }
            });
    }
    fn session(&mut self, _: &mut egui::Ui, _: &AppData, _: &mut Tts) {}
}
