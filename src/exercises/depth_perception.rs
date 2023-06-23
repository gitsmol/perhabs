use chrono::Duration;

use egui::{vec2, Align, Key, Vec2};
use tts::Tts;

use crate::shared::asset_loader::AppData;
use crate::shared::evaluation::Evaluation;
use crate::widgets;
use crate::widgets::evaluation::eval_config_widgets;
use crate::wm::sessionman::Exercise;

use self::anaglyph::Anaglyph;
mod anaglyph;

struct Session {
    active: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self { active: false }
    }
}

/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct DepthPerception {
    anaglyph: Anaglyph,
    calibrating: bool,
    evaluation: Evaluation<bool>,
    session: Session,
}

impl Default for DepthPerception {
    fn default() -> Self {
        Self {
            anaglyph: Anaglyph::default(),
            calibrating: false,
            evaluation: Evaluation::new(Duration::seconds(60), 60),
            session: Session::default(),
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl DepthPerception {
    /// Evaluate given answer
    fn evaluate_answer(&mut self) {}

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
            };
        });
    }

    /// Keeps track of answer, response, result progression.
    /// Record responses as f32:
    ///   - correct response = result 1.0
    ///   - incorrect or no response = result 0.0
    fn progressor(&mut self, ctx: &egui::Context) {
        // Repaint regularly to update timers!
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        self.read_keypress(ctx)
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
            ui.checkbox(&mut self.anaglyph.debug.show, "Debug");
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
        let menu_window = egui::Window::new("Vergence")
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(350., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        // There are three possible states:
        // - Finished session shows the evaluation scores
        // - Active session shows anaglyphs and keeps track of progression
        // - No session shows the exercise menu
        match self.session.active {
            true => match self.evaluation.is_finished() {
                true => {
                    menu_window.show(ctx, |ui| self.finished_screen(ui));
                }
                false => {
                    egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
                    self.progressor(ctx);
                }
            },
            false => {
                menu_window.show(ctx, |ui| {
                    self.ui(ui, appdata, tts);
                });
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

        // Display exercise configs
        egui::Grid::new("vergence_selector_grid")
            .num_columns(4)
            .show(ui, |ui| {
                if let Some(excconfig) = &appdata.excconfig {
                    for excercise in &excconfig.vergence {
                        ui.label(format!("{}", excercise.name));

                        for level in &excercise.levels {
                            if ui.button(&level.name).clicked() {
                                self.session.active = true;
                                self.evaluation.start();
                            }
                        }

                        ui.end_row();
                    }
                }
            });

        // Fill space
        ui.allocate_space(ui.available_size());

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
