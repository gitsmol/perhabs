use crate::shared::{AppData, Evaluation};
use crate::widgets::{self, menu_button};
use crate::wm::{Exercise, ExerciseType};
use chrono::Duration;
use egui::{vec2, Align, RichText, Vec2};
use rand::prelude::*;

use tts::{self, Tts};

use super::{numvec_to_string, ExerciseStage};

struct Answers {
    sequence: String,
    sequence_alpha: String,
    sequence_alpha_rev: String,
    sequence_rev: String,
}
impl Default for Answers {
    fn default() -> Self {
        Self {
            sequence: String::from("Press space/next to start."),
            sequence_alpha: String::new(),
            sequence_alpha_rev: String::new(),
            sequence_rev: String::new(),
        }
    }
}

/// Sequences
pub struct CogNumbers {
    seq_length: usize,
    session: ExerciseStage,
    answers: Answers,
    evaluation: Evaluation<bool>,
}

impl Default for CogNumbers {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            seq_length: 4,
            session: ExerciseStage::None,
            evaluation: Evaluation::new(Duration::try_seconds(240).unwrap_or_default(), 10),
        }
    }
}

impl CogNumbers {
    /// Keeps track of exercise progression
    fn progressor(&mut self) {
        // end exercise when evaluation is finished.
        if self.evaluation.is_finished() {
            self.session = ExerciseStage::Finished;
        };
    }

    fn next(&mut self, tts: &mut tts::Tts) {
        match self.session {
            ExerciseStage::Challenge => {
                self.evaluation.add_result(true);
                self.session = ExerciseStage::Result;
            }
            ExerciseStage::Result => {
                self.session = ExerciseStage::Challenge;
                self.pick_sequence();
                self.say(tts);
            }
            _ => (),
        }
    }

    fn say(&mut self, tts: &mut tts::Tts) {
        match tts.speak(&self.answers.sequence, false) {
            Ok(_) => debug!("TTS: Sentence spoken."),
            Err(e) => warn!("TTS error: {:?}", e),
        };
    }

    fn pick_sequence(&mut self) {
        let mut seq = vec![];
        if self.seq_length > 11 {
            return;
        }
        let mut rng = thread_rng();
        while seq.len() < self.seq_length {
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

    /// Review the evaluation.
    fn finished_screen(&mut self, ui: &mut egui::Ui) {
        widgets::evaluation::post_eval_widgets(
            ui,
            self.evaluation.average_score(),
            self.evaluation.reps_done(),
            self.evaluation.time_taken_as_string(),
        );

        // Close
        if ui.button("Close").clicked() {
            self.reset();
        }
    }

    fn read_keypress(&mut self, ctx: &egui::Context, tts: &mut tts::Tts) {
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.next(tts)
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.say(tts);
        }
    }
}

impl Exercise for CogNumbers {
    fn name(&self) -> &'static str {
        "Cognitive Numbers"
    }

    fn description(&self) -> &'static str {
        "Recall and reorder a sequence of numbers."
    }

    fn help(&self) -> &'static str {
        "This exercise uses your computers voice to say random numbers out loud. It is up to you to reorder the numbers in this sentence.

Each string of numbers will be shown in full, alongside with
- the numbers reversed
- the numbers ordered small to large
- the numbers ordered large to small

Try to rearrange the numbers and work your brain!"
    }

    fn excercise_type(&self) -> Vec<ExerciseType> {
        vec![ExerciseType::Cognitive]
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        let window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(350., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        match self.session {
            ExerciseStage::None => {
                window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            ExerciseStage::Finished => {
                window.show(ctx, |ui| self.finished_screen(ui));
            }
            _ => {
                self.read_keypress(ctx, tts);
                self.progressor();
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
        };

        if self.session != ExerciseStage::None {}
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        ui.label(self.help());
        ui.separator();

        // Show evaluation config
        widgets::evaluation::eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 600],
            [5, 60],
        );

        // Draw a menu in two columns
        let mut func = |i| {
            self.seq_length = i;
            self.evaluation.start();
            self.session = ExerciseStage::Result;
        };

        ui.columns(2, |col| {
            // Column 1 gets populated with at least half the buttons
            for i in 4..8 as usize {
                if menu_button(&mut col[0], None, None, format!("{i} numbers").as_str(), "")
                    .clicked()
                {
                    func(i);
                };
            }

            // Column 2 gets populated with the remaining buttons
            for i in 8..=10 as usize {
                if menu_button(&mut col[1], None, None, format!("{i} numbers").as_str(), "")
                    .clicked()
                {
                    func(i);
                };
            }
        });
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, tts: &mut Tts) {
        let spacer = ui.available_height() / 30.;

        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                *self = Default::default();
            };
            ui.label(format!(
                "Time remaining: {}",
                self.evaluation.time_remaining_as_string()
            ));
            ui.label(format!(
                "Reps remaining: {}",
                self.evaluation.reps_remaining()
            ));
        });

        ui.vertical_centered(|ui| {
            if self.session == ExerciseStage::Result {
                ui.add_space(spacer * 4.);

                ui.label("Sentence");
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
            };

            if self.session == ExerciseStage::Challenge {
                ui.add_space(spacer * 4.);
                ui.label("Try to reorder the numbers in your head.\nPress repeat (enter) to hear the numbers again.");
                ui.add_space(spacer * 9.);
            };

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
                self.next(tts);
            };

            ui.add_space(spacer);
            ui.label("Press space for next sequence. Press return to repeat sequence.");
        });
    }
}
