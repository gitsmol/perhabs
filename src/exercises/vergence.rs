use crate::windowman::{AppWin, View};
use chrono::{Local, Timelike};
use eframe::epaint::{self, CircleShape};
use egui::{emath, pos2, vec2, Color32, Frame, Pos2, Rect, Stroke};

use std::f32::consts::TAU;

pub struct Vergence {}

impl Default for Vergence {
    fn default() -> Self {
        Self {}
    }
}

impl Vergence {}

impl AppWin for Vergence {
    fn name(&self) -> &'static str {
        "Vergence"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, mut spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, spk));
    }
}

impl View for Vergence {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let desired_size = ui.available_width() * vec2(1., 1.);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);
        });
    }
}
