use crate::wm::sessionman::Exercise;
use crate::{exercises::numvec_to_string, shared::asset_loader::appdata::AppData};
use egui::RichText;
use rand::prelude::*;

use tts::{self, Tts};

#[derive(PartialEq, Eq)]
enum RoundStatus {
    Running,
    Finished,
    None,
}

struct Configuration {
    seq_length: usize,
    seq_show: bool,
    debug: bool,
}

struct Answers {
    generated: Vec<u32>,
    sequence: String,
    sequence_alpha: String,
    sequence_alpha_rev: String,
    sequence_rev: String,
}

impl Default for Answers {
    fn default() -> Self {
        Self {
            generated: vec![],
            sequence: String::from("No sequence."),
            sequence_alpha: String::new(),
            sequence_alpha_rev: String::new(),
            sequence_rev: String::new(),
        }
    }
}

/// A round is one round of delivering a sequence piece by piece.
/// Each round continues until the sequence is completed.
/// When the round is completed, set status to Finished.

#[derive(PartialEq, Eq)]
struct Round {
    status: RoundStatus,
    selected: [usize; 2], // what parts of the sequence are being delivered?
    timestamp: i64, // at what time should we stop displaying the current part of the sequence?
    step_secs: i64, // how many seconds is a step in a round?
}

impl Default for Round {
    fn default() -> Self {
        Self {
            status: RoundStatus::None,
            selected: [0, 1],
            step_secs: 3,
            timestamp: 0,
        }
    }
}

/// Sequences
pub struct MultiTasker {
    config: Configuration,
    session: bool,
    round: Round,
    answers: Answers,
}

impl Default for MultiTasker {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            config: Configuration {
                seq_length: 6,
                seq_show: false,
                debug: true,
            },
            session: false,
            round: Round::default(),
        }
    }
}

impl MultiTasker {
    fn say_part(&mut self, spk: &mut tts::Tts) -> () {
        let index = self.round.selected[1];
        let part = &self.answers.generated[index].to_string();
        match spk.speak(part, false) {
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

        self.answers.generated = seq.to_owned();
        self.answers.sequence = numvec_to_string(&seq);
        seq.reverse();
        self.answers.sequence_rev = numvec_to_string(&seq);
        seq.sort();
        self.answers.sequence_alpha = numvec_to_string(&seq);
        seq.reverse();
        self.answers.sequence_alpha_rev = numvec_to_string(&seq);
    }

    /// Progress round to next part of sequence. Set status to finished when running
    /// out of sequence length.
    fn next_step(&mut self) -> bool {
        // set timestamp in the future
        self.round.timestamp = chrono::Local::now().timestamp() + self.round.step_secs;

        // When starting the round, only set status to running and return false
        if self.round.status == RoundStatus::None {
            self.round.status = RoundStatus::Running;
            return true;
        };

        // When running out of sequence length, finish the round and return false
        // compare for the largest possible selected index with length of seq:
        if (self.round.selected[1] + 2) > self.answers.generated.len() {
            self.round.status = RoundStatus::Finished;

            return false;
        };

        // When round is running, move to the next part of the sequence and return true
        if self.round.status == RoundStatus::Running {
            self.round.selected[0] += 2;
            self.round.selected[1] += 2;
            return true;
        };

        // When in doubt, don't progress. (Because of hardcoded indexing,
        // running out of vec length is a potential panic.)
        false
    }

    fn next_round(&mut self) {
        self.pick_numbers();
        self.round = Round::default();
    }

    fn repeat_round(&mut self) {
        self.round = Round::default();
    }

    fn show_answers(&mut self, ui: &mut egui::Ui) {
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

impl Exercise for MultiTasker {
    fn name(&self) -> &'static str {
        "Multitasker"
    }

    fn description(&self) -> &'static str {
        "No description."
    }

    fn reset(&mut self) {
        *self = MultiTasker::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        if self.session {
            // always repaint while in session
            ctx.request_repaint();
            if self.round.status == RoundStatus::None {
                if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                    self.next_step();
                }
            }

            // listen for keyboard input at the end of a round
            if self.round.status == RoundStatus::Finished {
                if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                    self.next_round();
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.repeat_round();
                }
            }

            // show session
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        } else {
            egui::Window::new(self.name())
                .default_size((250.0, 250.0))
                .vscroll(false)
                .resizable(true)
                .show(ctx, |ui| self.ui(ui, appdata, tts));
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
                    ui.add(egui::Slider::new(&mut self.config.seq_length, 2..=10).step_by(2.));
                    ui.end_row();
                });

            if ui.button("Start session").clicked() {
                self.session = true; // start session
                self.pick_numbers(); // pick random numbers
            }
        });
    }
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, spk: &mut Tts) {
        // get timestamp
        let now = chrono::Local::now().timestamp();

        // Basic ui stuff
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = false;
                self.round = Round::default();
            };

            ui.checkbox(&mut self.config.debug, "Debug");

            if self.config.debug {
                ui.label("Sequence: ");
                ui.label(&self.answers.sequence);
                ui.add_space(10.);
                ui.label("Timer: ");
                ui.label((now - self.round.timestamp).to_string());
            }
        });
        ui.separator();

        // Round logic
        match self.round.status {
            RoundStatus::Running => {
                // If the current step in the round is done, progress
                if now >= self.round.timestamp {
                    // Only speak if the round progresses (returns true)
                    if self.next_step() {
                        self.say_part(spk);
                    }
                };

                // In the first second of a new step, show selected part of sequence
                if (now + self.round.step_secs - 1) < (self.round.timestamp) {
                    let index = self.round.selected[0];
                    let part = &self.answers.generated[index].to_string();
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 4.);
                        ui.heading(RichText::new(part).size(25.));
                    });
                };
            }
            RoundStatus::Finished => self.show_answers(ui),
            RoundStatus::None => {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 4.);
                    ui.heading(RichText::new("Press space to start.").size(25.));
                });
            }
        };
    }
}
