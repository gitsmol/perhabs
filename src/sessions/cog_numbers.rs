use crate::asset_loader::AppData;
use crate::sessionman::Exercise;
use crate::windowman::View;
use egui::RichText;
use rand::prelude::*;

use perhabs::numvec_to_string;

use tts::{self, Tts};

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

    fn pick_numbers(&mut self) -> () {
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

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        if !self.session.active {
            egui::Window::new(self.name())
                .default_size((250.0, 250.0))
                .vscroll(false)
                .resizable(true)
                .show(ctx, |ui| self.ui(ui, appdata, tts));
        }

        if self.session.active {
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.pick_numbers();
                self.say(tts);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.say(tts);
            }
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
    }
}

impl View for CogNumbers {
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
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = Session::default();
            };
        });

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 4.);

            ui.label("Sentence");
            ui.heading(RichText::new(&self.answers.sequence).size(25.));
            ui.add_space(20.);

            ui.label("Reversed");
            ui.label(RichText::new(&self.answers.sequence_rev).size(25.));
            ui.add_space(20.);

            ui.label("Alphabetical");
            ui.label(RichText::new(&self.answers.sequence_alpha).size(25.));
            ui.add_space(20.);

            ui.label("Alphabetical reversed");
            ui.label(RichText::new(&self.answers.sequence_alpha_rev).size(25.));
            ui.add_space(20.);

            ui.add_space(50.);
            ui.label("Press space for next sequence. Press return to repeat sequence.");
        });
    }
}
