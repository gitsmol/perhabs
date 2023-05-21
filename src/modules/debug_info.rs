use tts::Tts;

use crate::{
    modules::asset_loader::AppData,
    wm::windowman::{AppWin, View},
};

use super::timer::Timer;

pub struct DebugInfo {
    timer: Timer,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            timer: Timer::new(),
        }
    }
}

impl DebugInfo {
    fn asset_loader_debug(&self, ui: &mut egui::Ui, appdata: &AppData) {
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
}

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
        self.asset_loader_debug(ui, appdata);
    }
}
