use egui::vec2;

use crate::windowman::{AppWin, View};

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

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, spk));
    }
}

impl View for DebugInfo {
    fn ui(&mut self, ui: &mut egui::Ui, _spk: &mut tts::Tts) {
        let desired_size = vec2(ui.available_width(), ui.available_height());
        ui.label(format!("Avail width: {}", desired_size[0]));
        ui.label(format!("Avail height: {}", desired_size[1]));
    }
}
