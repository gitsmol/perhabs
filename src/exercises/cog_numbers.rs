use crate::modules::asset_loader::AppData;
use crate::wm::sessionman::Exercise;
use egui::{vec2, Align, RichText, Vec2};
use rand::prelude::*;

use tts::{self, Tts};

use super::numvec_to_string;

struct Session {
    active: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self { active: false }
    }
}

struct Configuration {
    seq_length: usize,
    seq_show: bool,
}

struct Answers {
    sequence: String,
    sequence_alpha: String,
    sequence_alpha_rev: String,
    sequence_rev: String,
}
impl Default for Answers {
    fn default() -> Self {
        Self {
            sequence: String::from("No sequence."),
            sequence_alpha: String::new(),
            sequence_alpha_rev: String::new(),
            sequence_rev: String::new(),
        }
    }
}

/// Sequences
pub struct CogNumbers {
    config: Configuration,
    session: Session,
    answers: Answers,
}

impl Default for CogNumbers {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            config: Configuration {
                seq_length: 4,
                seq_show: false,
            },
            session: Session::default(),
        }
    }
}

impl CogNumbers {
    fn say(&mut self, spk: &mut tts::Tts) -> () {
        match spk.speak(&self.answers.sequence, false) {
            Ok(_) => debug!("TTS: Sentence spoken."),
            Err(e) => warn!("TTS error: {:?}", e),
        };
    }

    fn pick_sequence(&mut self) -> () {
        let mut seq = vec![];
        if self.config.seq_length > 11 {
            return;
        }
        let mut rng = thread_rng();
        while seq.len() < self.config.seq_length {
            // this means no seq longer than 11 numbers (0..10)!
            let num = rng.gen_range(0..=10);
            if !seq.contains(&num) {
                seq.push(num);
            };
        }

        self.answers.sequence = numvec_to_string(&seq);
        seq.reverse();
        self.answers.sequence_rev = numvec_to_string(&seq);
        seq.sort();
        self.answers.sequence_alpha = numvec_to_string(&seq);
        seq.reverse();
        self.answers.sequence_alpha_rev = numvec_to_string(&seq);
    }
}

impl Exercise for CogNumbers {
    fn name(&self) -> &'static str {
        "CogNumbers"
    }

    fn description(&self) -> &'static str {
        "Recall and reorder a sequence of numbers."
    }

    fn reset(&mut self) {
        *self = CogNumbers::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        if !self.session.active {
            egui::Window::new(self.name())
                .anchor(
                    egui::Align2([Align::Center, Align::TOP]),
                    Vec2::new(0., 100.),
                )
                .fixed_size(vec2(350., 300.))
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .show(ctx, |ui| self.ui(ui, appdata, tts));
        }

        if self.session.active {
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.pick_sequence();
                self.say(tts);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.say(tts);
            }
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        // normal stuff
        ui.vertical(|ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Show sequence");
                    ui.checkbox(&mut self.config.seq_show, "");
                    ui.end_row();
                    ui.label("Sequence length");
                    ui.add(egui::Slider::new(&mut self.config.seq_length, 1..=10));
                    ui.end_row();
                });

            if ui.button("Start session").clicked() {
                self.session.active = true;
            }

            if self.config.seq_show {
                ui.separator();
                egui::Grid::new("answer_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Sequence");
                        ui.heading(&self.answers.sequence.to_string());
                        ui.end_row();

                        ui.label("Reversed");
                        ui.label(&self.answers.sequence_rev);
                        ui.end_row();

                        ui.label("Alphabetical");
                        ui.label(&self.answers.sequence_alpha);
                        ui.end_row();

                        ui.label("Alphabetical reversed");
                        ui.label(&self.answers.sequence_alpha_rev);
                        ui.end_row();
                    });
            }
        });
    }
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, tts: &mut Tts) {
        let spacer = ui.available_height() / 30.;
        if ui.button("Close").clicked() {
            self.session = Session::default();
        };

        ui.vertical_centered(|ui| {
            ui.add_space(spacer * 4.);

            ui.label("Sequence");
            ui.heading(RichText::new(&self.answers.sequence).size(25.));
            ui.add_space(spacer);

            ui.label("Reversed");
            ui.label(RichText::new(&self.answers.sequence_rev).size(25.));
            ui.add_space(spacer);

            ui.label("Alphabetical");
            ui.label(RichText::new(&self.answers.sequence_alpha).size(25.));
            ui.add_space(spacer);

            ui.label("Alphabetical reversed");
            ui.label(RichText::new(&self.answers.sequence_alpha_rev).size(25.));
            ui.add_space(spacer);

            ui.add_space(spacer * 2.);

            if ui
                .add_sized(vec2(spacer * 4., spacer * 2.), egui::Button::new("Repeat"))
                .clicked()
            {
                self.say(tts);
            };

            ui.add_space(spacer / 4.);

            if ui
                .add_sized(vec2(spacer * 4., spacer * 2.), egui::Button::new("Next"))
                .clicked()
            {
                self.pick_sequence();
                self.say(tts);
            };

            ui.add_space(spacer);
            ui.label("Press space for next sequence. Press return to repeat sequence.");
        });
    }
}
