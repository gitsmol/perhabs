use crate::shared::{AppData, Evaluation, Timer};
use crate::widgets::{self, menu_button};
use crate::wm::{Exercise, ExerciseType};
use chrono::Duration;

use egui::Pos2;
use egui::{vec2, Align, Vec2};
use rand::prelude::*;
mod session;

use tts::{self, Tts};

use crate::exercises::shared::grid::Grid;
use crate::exercises::ExerciseStage;

#[derive(Default)]
struct Answers {
    sequence: Vec<Pos2>,
    response: Vec<Pos2>,
}

/// Sequences
pub struct NumberedSquares {
    seq_length: usize,
    stage: ExerciseStage,
    answers: Answers,
    grid: Grid,
    grid_size: usize,
    challenge_ms: i64,
    challenge_timer: Timer,
    response_ms: i64,
    response_timer: Timer,
    result_timer: Timer,
    result_ms: i64,
    evaluation: Evaluation<bool>,
}

impl Default for NumberedSquares {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            seq_length: 4,
            stage: ExerciseStage::None,
            grid: Grid::new(),
            grid_size: 10,
            challenge_ms: 2000,
            challenge_timer: Timer::new(),
            response_ms: 10_000,
            response_timer: Timer::new(),
            result_timer: Timer::new(),
            result_ms: 2000,
            evaluation: Evaluation::new(Duration::try_seconds(240).unwrap_or_default(), 10),
        }
    }
}

impl NumberedSquares {
    /// Keeps track of exercise progression
    fn progressor(&mut self) {
        // end exercise when evaluation is finished.
        if self.evaluation.is_finished() {
            self.stage = ExerciseStage::Finished;
        };

        // if we are still running, progress through exercise stages
        match self.stage {
            // Showing numbers on squares
            ExerciseStage::Challenge => {
                if self.challenge_timer.is_finished() {
                    self.challenge_timer.reset();
                    self.response_timer
                        .set(Duration::try_milliseconds(self.response_ms).unwrap_or_default());
                    self.stage = ExerciseStage::Response;
                }
            }
            // Allowing user input
            ExerciseStage::Response => {
                let go_next = self.response_timer.is_finished()
                    || self.answers.response.len() == self.answers.sequence.len();
                if go_next {
                    self.response_timer.reset();
                    self.result_timer
                        .set(Duration::try_milliseconds(self.result_ms).unwrap_or_default());
                    self.stage = ExerciseStage::Result;
                }
            }
            // Showing correct/incorrect
            ExerciseStage::Result => {
                if self.result_timer.is_finished() {
                    self.next();
                }
            }
            _ => (),
        };
    }

    /// Evaluate response, store result, move on to next challenge
    fn next(&mut self) {
        self.evaluation.add_result(self.evaluate_response());
        self.gen_sequence();
        self.answers.response.clear();
        self.stage = ExerciseStage::Challenge;
        self.challenge_timer
            .set(Duration::try_milliseconds(self.challenge_ms).unwrap_or_default())
    }

    fn evaluate_response(&self) -> bool {
        if self.answers.response == self.answers.sequence {
            true
        } else {
            false
        }
    }

    /// Generates a sequence of valid and unique positions on the grid
    fn gen_sequence(&mut self) {
        self.answers.sequence.clear();
        let mut rng = thread_rng();
        let all_coords: Vec<Pos2> = self
            .grid
            .get_all_coords(self.grid_size)
            .into_iter()
            .flatten()
            .collect();

        while self.answers.sequence.len() < self.seq_length {
            // this means no seq longer than 11 numbers (0..10)!
            let num = rng.gen_range(0..all_coords.len());
            if let Some(pos) = all_coords.get(num) {
                if !self.answers.sequence.contains(pos) {
                    self.answers.sequence.push(*pos);
                };
            };
        }
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

    fn draw_debug_info(&self, ui: &mut egui::Ui) {
        // debug info in top bar
        ui.horizontal(|ui| {
            ui.label(format!("Sequence: {:?}", self.answers.sequence));
            ui.label(format!("Reponse: {:?}", self.answers.response));
            ui.label(format!("Stage: {:?}", self.stage));
            ui.add_space(10.0);
            ui.label(format!(
                "Challenge timer: {:?}",
                self.challenge_timer.remaining().to_string()
            ));
            ui.label(format!(
                "Response timer: {:?}",
                self.response_timer.remaining().to_string()
            ));
            ui.label(format!(
                "Result timer: {:?}",
                self.result_timer.remaining().to_string()
            ));
        });
    }
}

impl Exercise for NumberedSquares {
    fn name(&self) -> &'static str {
        "Numbered Squares"
    }

    fn description(&self) -> &'static str {
        "Recall the order of the numbered squares."
    }

    fn help(&self) -> &'static str {
        "Todo!"
    }

    fn excercise_type(&self) -> Vec<ExerciseType> {
        vec![ExerciseType::Cognitive, ExerciseType::Visual]
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        // Define menu window
        let window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(350., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        // If we aren't showing the menu or the finished screen, we're in a session.
        match self.stage {
            ExerciseStage::None => {
                window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            ExerciseStage::Finished => {
                window.show(ctx, |ui| self.finished_screen(ui));
            }
            _ => {
                self.progressor();
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
        };

        if self.stage != ExerciseStage::None {}
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
            self.gen_sequence();
            self.stage = ExerciseStage::Challenge;
            self.challenge_timer
                .set(Duration::try_milliseconds(self.challenge_ms).unwrap_or_default())
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

    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // Always check progression
        self.progressor();

        if appdata.debug {
            self.draw_debug_info(ui);
        };

        // session menu bar
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

        // Draw grid
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            if appdata.debug {
                self.draw_debug(ui);
                return;
            }
            self.draw_session(ui);
        });
    }
}
