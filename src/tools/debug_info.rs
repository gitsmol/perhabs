use egui::vec2;
use tts::Tts;

use crate::{
    shared::asset_loader::AppData,
    wm::windowman::{AppWin, View},
};

pub struct DebugInfo {}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {}
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
        ui.separator();
        ui.heading("Debug messages.");
        for msg in &appdata.debug_messages {
            ui.label(msg);
        }
    }
}

impl AppWin for DebugInfo {
    fn name(&self) -> &'static str {
        "Debugging info"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, appdata: &AppData, tts: &mut Tts) {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .fixed_size(vec2(500., 500.))
            .show(ctx, |ui| self.ui(ui, appdata, tts));
    }
}

impl View for DebugInfo {
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _tts: &mut Tts) {
        self.asset_loader_debug(ui, appdata);
    }
}
