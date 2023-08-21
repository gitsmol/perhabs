use chrono::Duration;

use egui::{Align, Key, Vec2};
use tts::Tts;

use crate::exercises::Direction;

use crate::shared::asset_loader::exercise_config::vergence::VergenceConfig;
use crate::shared::AppData;
use crate::shared::Evaluation;
use crate::widgets;
use crate::widgets::evaluation::eval_config_widgets;
use crate::widgets::exercise_config_menu::exercise_config_menu_multicol;
use crate::wm::Exercise;

use self::anaglyph::Anaglyph;
mod anaglyph;

struct Session {
    active: bool,
    answer_thresh_success: bool,
    answer_thresh_fail: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            active: false,
            answer_thresh_success: false,
            answer_thresh_fail: false,
        }
    }
}

/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct Vergence {
    anaglyph: Anaglyph,
    calibrating: bool,
    evaluation: Evaluation<bool>,
    session: Session,
    step: isize,
}

impl Default for Vergence {
    fn default() -> Self {
        Self {
            anaglyph: Anaglyph::default(),
            calibrating: false,
            evaluation: Evaluation::new(Duration::seconds(60), 60),
            session: Session::default(),
            step: 0,
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl Vergence {
    /// Evaluate given answer
    fn evaluate_answer(&mut self, a: Direction) {
        // If the answer is correct, add true to the results vec.
        // If the previous answer was also correct (indicated by the answer threshold),
        // increase the difficulty of the exercise.
        // If the previous answer was not correct, set the answer threshold to true.
        if a == self.anaglyph.focal_position {
            // Add result to evaluation
            self.evaluation.add_result(true);
            // Any correct answer invalidates the failure streak.
            self.session.answer_thresh_fail = false;
            match self.evaluation.show_results().last() {
                Some(prev_val) => {
                    if prev_val == &true && self.session.answer_thresh_success == true {
                        self.session.answer_thresh_success = false;
                        self.anaglyph.background_offset += self.step;
                    }
                    if prev_val == &true && self.session.answer_thresh_success == false {
                        self.session.answer_thresh_success = true
                    }
                    if prev_val == &false {
                        self.session.answer_thresh_success = false
                    }
                }
                None => (),
            }
        }

        // If the answer is incorrect, add false to the results vec.
        // If the previous answer was also incorrect (indicated by the answer threshold),
        // reset the difficulty of the exercise and set the answer_threshold to false.
        // If the previous answer was correct, set the answer_threshhold to true.
        if a != self.anaglyph.focal_position {
            // Add result to evaluation
            self.evaluation.add_result(false);
            // Any failure invalidates the success streak.
            self.session.answer_thresh_success = false;
            match self.evaluation.show_results().last() {
                Some(prev_val) => {
                    if prev_val == &false && self.session.answer_thresh_fail == true {
                        self.session.answer_thresh_fail = false;
                        self.anaglyph.background_offset = 0;
                    } else {
                        self.session.answer_thresh_fail = true
                    }
                }
                None => (),
            }
        }
        // create arrays for a new anaglyph
        self.anaglyph.initialize();
    }

    fn read_keypress(&mut self, ctx: &egui::Context) -> Option<Direction> {
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            return Some(Direction::Up);
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            return Some(Direction::Down);
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            return Some(Direction::Left);
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            return Some(Direction::Right);
        };
        None
    }

    /// Keeps track of answer, response, result progression.
    /// Record responses as f32:
    ///   - correct response = result 1.0
    ///   - incorrect or no response = result 0.0
    fn progressor(&mut self, ctx: &egui::Context) {
        // Repaint regularly to update timers!
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        if let Some(answer) = self.read_keypress(ctx) {
            self.evaluate_answer(answer);
        }
    }
}

// ***********
// UI
// ***********
impl Vergence {
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

impl Exercise for Vergence {
    fn name(&self) -> &'static str {
        "Vergence"
    }

    fn description(&self) -> &'static str {
        "Train your eyes to diverge and converge. Requires glasses in two different colors."
    }

    fn reset(&mut self) {
        // Remember color calibrations
        let tmp_color = self.anaglyph.color.clone();
        *self = Default::default();
        self.anaglyph.color = tmp_color;
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        let menu_window = egui::Window::new("Vergence")
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
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

        let mut func = |config_level: &VergenceConfig| {
            self.anaglyph.initialize();
            self.step = config_level.step;
            self.anaglyph.pixel_size = config_level.pixel_size;
            self.session.active = true;
            self.evaluation.start();
        };

        // Display exercise configs
        if let Some(excconfig) = &appdata.excconfig {
            egui::ScrollArea::new([false, true])
                .max_height(400.)
                .drag_to_scroll(true)
                .show(ui, |ui| {
                    ui.heading("Convergence");
                    if let Some(config) = exercise_config_menu_multicol::<VergenceConfig>(
                        ui,
                        &excconfig.convergence,
                        3,
                    ) {
                        func(config)
                    };

                    ui.heading("Divergence");
                    if let Some(config) = exercise_config_menu_multicol::<VergenceConfig>(
                        ui,
                        &excconfig.divergence,
                        3,
                    ) {
                        func(config)
                    };
                });
        };

        // Add some space and show calibration button
        ui.add_space(20.);

        if ui.button("Calibrate").clicked() {
            self.calibrating = true
        }
    }

    /// The session window showing anaglyphs
    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        self.ui_controls(ui);
        if appdata.debug {
            self.anaglyph.debug_controls(ui);
        }

        self.anaglyph.draw(ui);
    }
}
