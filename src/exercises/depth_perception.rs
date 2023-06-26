use chrono::Duration;

use egui::{Align, Key, Vec2};
use tts::Tts;

use crate::shared::asset_loader::exercise_config::depth_perception::DepthPerceptionExercise;
use crate::shared::asset_loader::exercise_config::ExerciseConfig;
use crate::shared::asset_loader::AppData;
use crate::shared::evaluation::Evaluation;
use crate::widgets;
use crate::widgets::evaluation::eval_config_widgets;
use crate::widgets::exercise_config_menu::exercise_config_menu;
use crate::wm::sessionman::Exercise;

use self::anaglyph::Anaglyph;
mod anaglyph;

use super::SessionStatus;

/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct DepthPerception {
    anaglyph: Anaglyph,
    calibrating: bool,
    evaluation: Evaluation<f32>,
    session: SessionStatus,
}

impl Default for DepthPerception {
    fn default() -> Self {
        Self {
            anaglyph: Anaglyph::default(),
            calibrating: false,
            evaluation: Evaluation::new(Duration::seconds(60), 60),
            session: SessionStatus::None,
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
        if self.session == SessionStatus::Response {
            // Repaint regularly to update timers!
            // NB this also sets bounds on the timer precision.
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            self.read_keypress(ctx);
            if self.evaluation.is_finished() {
                self.session = SessionStatus::Finished;
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

    fn calibrate(&mut self, ui: &mut egui::Ui) {
        widgets::calibrate_anaglyph::calibrate(&mut self.anaglyph.color, ui);

        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                self.anaglyph = Anaglyph::default();
                self.calibrating = false
            }
            if ui.button("Save and close").clicked() {
                self.calibrating = false
            }
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
            // .fixed_size(vec2(350., 300.))
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
            SessionStatus::Response => {
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
            SessionStatus::Finished => {
                menu_window.show(ctx, |ui| self.finished_screen(ui));
            }
            _ => {
                menu_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
        };
    }

    /// The exercise menu
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        if self.calibrating {
            self.calibrate(ui);
            return;
        }

        ui.label("This excercise shows a square. Inside the square is a diamond. Press the arrow key to indicate where you see the diamond in the square: left, right, up or down.");
        ui.separator();

        // Display the evaluation config
        eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
        );

        let mut func = |config: &DepthPerceptionExercise| {
            self.session = SessionStatus::Response;
            self.anaglyph.config = config.clone();
            self.evaluation.start();
            self.anaglyph.next();
        };

        // Display exercise configs
        if let Some(config) = &appdata.excconfig {
            if let Some(config) =
                exercise_config_menu::<DepthPerceptionExercise>(ui, &config.depth_perception)
            {
                func(config)
            };
        }

        // Add some space
        ui.add_space(ui.available_height() * 0.05);

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
