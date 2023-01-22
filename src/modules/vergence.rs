use egui::Key;
use perhabs::Direction;

use crate::modules::exercises::anaglyph::Anaglyph;
use crate::windowman::{AppWin, View};

/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct Vergence {
    anaglyph: Anaglyph,
}

impl Default for Vergence {
    fn default() -> Self {
        Self {
            anaglyph: Anaglyph::default(),
        }
    }
}

impl AppWin for Vergence {
    fn name(&self) -> &'static str {
        "Vergence"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, _spk: &mut tts::Tts) {
        if open == &true {
            egui::CentralPanel::default().show(ctx, |ui| self.ui(ui, _spk));
            self.read_keypress(ctx);
        }
    }
}

impl View for Vergence {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts) {
        self.debug_controls(ui);

        self.anaglyph.draw(ui);
    }
}

impl Vergence {
    fn read_keypress(&mut self, ctx: &egui::Context) {
        let mut eval = |a: Direction| {
            if a == self.anaglyph.focal_position {
                self.anaglyph.session.count += 1;
                self.anaglyph.session.results.push(true);
                self.anaglyph.initialize();
            }
        };

        if ctx.input().key_pressed(Key::ArrowUp) {
            eval(Direction::Up)
        };
        if ctx.input().key_pressed(Key::ArrowDown) {
            eval(Direction::Down)
        };
        if ctx.input().key_pressed(Key::ArrowLeft) {
            eval(Direction::Left)
        };
        if ctx.input().key_pressed(Key::ArrowRight) {
            eval(Direction::Right)
        };
    }

    fn debug_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::Checkbox::new(
                &mut self.anaglyph.debug.draw_left,
                "Left",
            ));
            ui.add(egui::Checkbox::new(
                &mut self.anaglyph.debug.draw_right,
                "Right",
            ));
            ui.add(egui::Checkbox::new(
                &mut self.anaglyph.debug.focal_mark,
                "Focal mark",
            ));
            ui.label(&self.anaglyph.debug.size_info);
            ui.spacing();
            ui.label(format!("Count: {}", &self.anaglyph.session.count));
            ui.label(format!("Start time: {}", &self.anaglyph.session.start_time));
        });
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.anaglyph.pixel_size, 1..=10).suffix("pixel size"));
            ui.add(
                egui::Slider::new(&mut self.anaglyph.grid_size, 10..=150).suffix("anaglyph size"),
            );
            if ui
                .add(
                    egui::Slider::new(&mut self.anaglyph.background_offset, 0..=30)
                        .suffix("bg_offset size"),
                )
                .changed()
            {
                self.anaglyph.initialize()
            };
            if ui
                .add(
                    egui::Slider::new(&mut self.anaglyph.focal_offset, 0..=10)
                        .suffix("focal_offset"),
                )
                .changed()
            {
                self.anaglyph.initialize()
            };
        });
    }
}
