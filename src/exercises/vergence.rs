use crate::windowman::{AppWin, View};
use egui::{color_picker::show_color, vec2, Color32};

use ndarray::Array2;

pub struct Vergence {
    matrix_size: u32,
    matrix: Array2<u8>,
}

impl Vergence {}

impl Default for Vergence {
    fn default() -> Self {
        Self {
            matrix_size: 10,
            matrix: Array2::<u8>::zeros((10, 10)),
        }
    }
}

impl AppWin for Vergence {
    fn name(&self) -> &'static str {
        "Vergence"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, mut spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, &mut spk));
    }
}

impl View for Vergence {
    fn ui(&mut self, ui: &mut egui::Ui, mut spk: &mut tts::Tts) {
        ui.heading("Vergence test");
        let color = Color32::BLUE;
        let size = vec2(10.0, 10.0);
        for i in self.matrix.rows() {
            for x in self.matrix.columns() {
                show_color(ui, color, size);
            }
        }
    }
}
