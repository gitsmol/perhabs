use chrono::Duration;

use egui::{Align, Key, Vec2};
use tts::Tts;

use crate::shared::asset_loader::exercise_config::depth_perception::DepthPerceptionConfig;

use crate::shared::AppData;
use crate::shared::Evaluation;
use crate::widgets;
use crate::widgets::evaluation::eval_config_widgets;
use crate::widgets::exercise_config_menu::exercise_config_menu_multicol;
use crate::wm::Exercise;

use self::anaglyph::Anaglyph;
mod anaglyph;

use super::ExerciseStatus;

/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct DepthPerception {
    anaglyph: Anaglyph,
    calibrating: bool,
    evaluation: Evaluation<f32>,
    session: ExerciseStatus,
}

impl Default for DepthPerception {
    fn default() -> Self {
        Self {
            anaglyph: Anaglyph::default(),
            calibrating: false,
            evaluation: Evaluation::new(Duration::seconds(60), 60),
            session: ExerciseStatus::None,
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl DepthPerception {
    /// Evaluate given answer. Records responses as f32:
    ///   - correct response = result 1.0
    ///   - incorrect or no response = result 0.0
    fn evaluate_answer(&mut self) {
        if self.anaglyph.arrow_position == self.anaglyph.target_index {
            self.evaluation.add_result(1.0)
        } else {
            self.evaluation.add_result(0.0)
        }
    }

    /// Move the indicator arrow and give an answer by pressing enter.
    fn read_keypress(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // move left
            if i.key_pressed(Key::ArrowLeft) && self.anaglyph.arrow_position > 0 {
                self.anaglyph.arrow_position -= 1;
            };
            // move right
            // note the correction for counting from zero
            if i.key_pressed(Key::ArrowRight)
                && self.anaglyph.arrow_position < self.anaglyph.circles - 1
            {
                self.anaglyph.arrow_position += 1;
            };

            // press enter to give answer
            if i.key_pressed(Key::Enter) {
                self.evaluate_answer();
                self.anaglyph.next();
            };
        });
    }

    /// Keeps track of answer, response, result progression.
    fn progressor(&mut self, ctx: &egui::Context) {
        // The only change is from Response to Finished. All other status changes
        // are forced by the user.
        if self.session == ExerciseStatus::Response {
            // Repaint regularly to update timers!
            // NB this also sets bounds on the timer precision.
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            self.read_keypress(ctx);
            if self.evaluation.is_finished() {
                self.session = ExerciseStatus::Finished;
            }
        }
    }
}

// ***********
// UI
// ***********
impl DepthPerception {
    /// Basic controls during a session
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                // Reset the whole exercise.
                self.reset();
            };
            ui.label(format!(
                "Time remaining: {}:{:02}",
                self.evaluation.time_remaining().num_minutes(),
                self.evaluation.time_remaining().num_seconds()
            ));
            ui.label(format!(
                "Reps remaining: {}",
                self.evaluation.reps_remaining()
            ));

            ui.add_space(ui.available_width() - 100.0);
        });
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
}

// ***********
// Exercise trait
// ***********
impl Exercise for DepthPerception {
    fn name(&self) -> &'static str {
        "Depth perception"
    }

    fn description(&self) -> &'static str {
        "Learn to see (minor) differences in object depth."
    }

    fn reset(&mut self) {
        // Remember color calibrations
        let tmp_color = self.anaglyph.color.clone();
        *self = DepthPerception::default();
        self.anaglyph.color = tmp_color;
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        let menu_window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .resizable(false)
            .movable(false)
            .collapsible(false);

        // Keep track of progress
        self.progressor(ctx);

        // There are three possible states:
        // - Response means we are showing the exercise and taking responses
        // - Finished session shows the evaluation scores
        // - Anything else shows the exercise menu
        match self.session {
            ExerciseStatus::Response => {
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
            ExerciseStatus::Finished => {
                menu_window.show(ctx, |ui| self.finished_screen(ui));
            }
            _ => {
                menu_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
        };
    }

    /// The exercise menu
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // Calibration guard clause
        if self.calibrating {
            widgets::calibrate_anaglyph::calibrate(
                ui,
                &mut self.anaglyph.color,
                &mut self.calibrating,
            );
            return;
        }

        ui.label("This excercise shows a square. Inside the square is a diamond. Press the arrow key to indicate where you see the diamond in the square: left, right, up or down.");
        ui.separator();

        // Display the evaluation config
        eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 120],
            [30, 120],
        );

        let mut func = |config: &DepthPerceptionConfig| {
            self.session = ExerciseStatus::Response;
            self.anaglyph.config = config.clone();
            self.evaluation.start();
            self.anaglyph.next();
        };

        // Display exercise configs
        if let Some(config) = &appdata.excconfig {
            if let Some(config) = exercise_config_menu_multicol::<DepthPerceptionConfig>(
                ui,
                &config.depth_perception,
                2,
            ) {
                func(config)
            };
        }

        // Add some space and show calibration button
        ui.add_space(20.);

        if ui.button("Calibrate").clicked() {
            self.calibrating = true
        }
    }

    /// The session window showing anaglyphs
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        self.ui_controls(ui);
        self.anaglyph.draw(ui);
    }
}
