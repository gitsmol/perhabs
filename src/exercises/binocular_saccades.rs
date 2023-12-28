use crate::exercises::Direction;
use crate::shared::asset_loader::exercise_config::vergence::VergenceConfig;
use crate::shared::Anaglyph;
use crate::shared::AppData;
use crate::widgets::evaluation::eval_config_widgets;
use crate::widgets::exercise_config_menu::exercise_config_menu_multicol;
use crate::widgets::{self};
use crate::{
    wm::Exercise,
    {shared::Evaluation, shared::Timer},
};
use chrono::Duration;
use egui::{pos2, vec2, Align, Frame, Key, Pos2, Vec2};
use rand::{seq::SliceRandom, Rng};

use super::ExerciseStatus;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BinoSaccades {
    session_status: ExerciseStatus,
    anaglyph: Anaglyph,
    anaglyph_pos: Option<Pos2>,
    answer: Option<Direction>, // The right answer is the direction of the arrow
    response: Option<Direction>, // The given response is a direction
    answer_timeout_timer: Timer,
    answer_timeout_ms: i64,
    evaluation: Evaluation<f32>,
}

impl Default for BinoSaccades {
    fn default() -> Self {
        Self {
            session_status: ExerciseStatus::None,
            anaglyph: Anaglyph::default(),
            anaglyph_pos: None,
            answer: None,
            response: None,
            answer_timeout_timer: Timer::new(),
            answer_timeout_ms: 1000,
            evaluation: Evaluation::new(Duration::seconds(60), 60),
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl BinoSaccades {
    fn anaglyph_painter(&mut self, ui: &mut egui::Ui, appdata: &AppData) {
        match self.anaglyph.draw(ui) {
            Ok(_) => (),
            Err(e) => {
                appdata.error_tx.send(e.to_string());
            }
        };
    }

    /// Randomly position an arrow pointing in a random direction.
    fn new_anaglyph_pos(&mut self) {
        let mut rng = rand::thread_rng();

        if let Some(direction) = vec![
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ]
        .choose(&mut rng)
        {
            self.answer = Some(*direction);
            self.anaglyph.focal_position = *direction;
        }

        let x: f32 = rng.gen_range(0.15..0.85);
        let y: f32 = rng.gen_range(0.15..0.85);
        self.anaglyph_pos = Some(pos2(x, y));
        self.anaglyph.screen_offset = Some(pos2(x, y));
    }

    /// Keeps track of answer, response, result progression.
    /// This exercise is only every in Response mode:
    /// - constantly display new arrows until timeout or user input
    /// - record responses:
    ///   - correct response = result 1.0
    ///   - incorrect or no response = result 0.0
    fn progressor(&mut self, ctx: &egui::Context) {
        // Repaint regularly to update timers!
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // When the evaluation time is up or number of reps is reached, stop immediately.
        if self.evaluation.is_finished() {
            self.session_status = ExerciseStatus::Finished;
        }

        match self.session_status {
            // This exercise is always in response mode.
            ExerciseStatus::Response => {
                // Setup and display answer
                // If no anaglyph is visible, create new anaglyph and set answer timeout timer
                if let None = self.answer {
                    self.new_anaglyph_pos();
                    self.anaglyph.initialize();
                    self.answer_timeout_timer
                        .set(Duration::milliseconds(self.answer_timeout_ms));
                }

                // Continously allow response input
                self.read_keypress(ctx);

                // After the answer timeout is up, delete the arrow and evaluate response.
                // This will trigger a new arrow with timeout.
                if self.answer_timeout_timer.is_finished() {
                    self.next();
                }
            }
            // None and Finished don't trigger progression.
            _ => (),
        };
    }

    fn next(&mut self) {
        self.evaluation.add_result(self.evaluate_response());
        self.answer = None;
        self.response = None;
    }

    /// Determine correctness of response.
    /// Correct = 1.0
    /// Incorrect = 0.0
    fn evaluate_response(&self) -> f32 {
        // Only return 1.0 (correct) if current response matches current answer.
        if let Some(direction) = self.response {
            if let Some(answer) = self.answer {
                if direction == answer {
                    return 1.0;
                }
            }
        }

        // default
        0.0
    }

    /// Read arrow keys and register response.
    fn read_keypress(&mut self, ctx: &egui::Context) {
        let mut eval = |response: Direction| {
            self.response = Some(response);
            self.next();
        };

        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            eval(Direction::Up)
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            eval(Direction::Down)
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            eval(Direction::Left)
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            eval(Direction::Right)
        };
    }
}

// ***********
// UI
// ***********
impl BinoSaccades {
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

impl Exercise for BinoSaccades {
    fn name(&self) -> &'static str {
        "Binocular scanning (Saccades)"
    }

    fn description(&self) -> &'static str {
        "Quickly scan the screen and respond. Requires anaglyph glasses!"
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut tts::Tts) {
        let menu_window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(500., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        match self.session_status {
            // Default shows the menu
            ExerciseStatus::None => {
                menu_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            // After an evaluation show the review
            ExerciseStatus::Finished => {
                menu_window.show(ctx, |ui| self.finished_screen(ui));
            }
            // Any other status means we are in session.
            _ => {
                // Keep track of progression of session
                self.progressor(ctx);
                // Show session panel
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut tts::Tts) {
        // Display the evaluation config
        eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 120],
            [30, 120],
        );

        ui.horizontal(|ui| {
            if ui.button("Quick").clicked() {
                self.answer_timeout_ms = 500;
            }
            if ui.button("Normal").clicked() {
                self.answer_timeout_ms = 1000;
            }
            if ui.button("Slow").clicked() {
                self.answer_timeout_ms = 1500;
            }
        });

        // Display all exercise configs
        let mut func = |exercise: &VergenceConfig| {
            self.anaglyph.initialize();
            self.anaglyph.pixel_size = exercise.pixel_size;
            self.session_status = ExerciseStatus::Response;
            self.evaluation.start();
        };

        // Display exercise configs
        if let Some(config) = &appdata.excconfig {
            if let Some(config) =
                exercise_config_menu_multicol::<VergenceConfig>(ui, &config.convergence, 3)
            {
                func(config)
            };
        }
    }

    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut tts::Tts) {
        self.ui_controls(ui);
        Frame::dark_canvas(ui.style()).show(ui, |ui| self.anaglyph_painter(ui, appdata));
    }
}
