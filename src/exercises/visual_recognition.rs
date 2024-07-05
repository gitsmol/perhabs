use crate::exercises::Direction;
use crate::shared::asset_loader::exercise_config::visual_recognition::VisRecognitionConfig;
use crate::widgets;
use crate::widgets::evaluation::eval_config_widgets;
use crate::widgets::exercise_config_menu::exercise_config_menu;

use crate::wm::ExerciseType;
use crate::{
    wm::Exercise,
    {shared::AppData, shared::Evaluation, shared::Timer},
};
use chrono::Duration;
use egui::{emath, pos2, vec2, Align, Color32, Frame, Key, Rect, Vec2};
use rand::seq::SliceRandom;
use std::iter::zip;

use super::ExerciseStage;

/// Visual exercise to train quick recognition and retention of shapes.
/// Draws a number of arrows in the middle of the screen. The arrows remain visible
/// for a short period of time. Then player presses the arrow keys to indicate the
/// sequence of arrows they have seen.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VisRecognition {
    session_status: ExerciseStage,
    exercise_params: VisRecognitionConfig,
    answer: Vec<Direction>,   // The correct answer: a sequence of directions
    timer: Timer,             // Keeps track of timeouts
    response: Vec<Direction>, // The response given by the player
    evaluation: Evaluation<f32>,
}

impl Default for VisRecognition {
    fn default() -> Self {
        Self {
            session_status: ExerciseStage::None,
            exercise_params: VisRecognitionConfig::default(),
            answer: vec![],
            timer: Timer::new(),
            response: vec![],
            evaluation: Evaluation::new(Duration::try_seconds(60).unwrap_or_default(), 60),
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl VisRecognition {
    /// Paints the arrow shapes
    fn arrow_painter(&self, ui: &mut egui::Ui) {
        // Determine size of drawing surface and aspect ratio
        let (_id, rect) = ui.allocate_space(ui.available_size_before_wrap());
        let aspect = rect.width() / rect.height();
        // Create a transform mapping the available space on a rectangle,
        // taking aspect ratio into account
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=aspect, 0.0..=1.0), rect);

        // Determine where to start drawing.
        // Take aspect ratio into account!
        let center = pos2(aspect / 2.0, 0.5);
        let measure = self.exercise_params.arrow_size as f32 / 3. * 0.02;
        let margin = measure * 2.5;
        let x_start = center - vec2(self.answer.len() as f32 / 2. * (measure * 2.), 0.);

        // Paint the answer
        if self.session_status == ExerciseStage::Challenge
            || self.session_status == ExerciseStage::Result
        {
            for (i, direction) in self.answer.iter().enumerate() {
                let pos = x_start + vec2(i as f32 * margin, 0.);
                let shapes = widgets::arrow_shape(
                    pos,
                    self.exercise_params.arrow_size as f32,
                    direction,
                    to_screen,
                    Color32::LIGHT_GREEN,
                );
                ui.painter().add(shapes);
            }
        }

        // Paint the response
        // Draw response arrows below the given sequence.
        if self.session_status == ExerciseStage::Response
            || self.session_status == ExerciseStage::Result
        {
            for (i, direction) in self.response.iter().enumerate() {
                let pos = x_start + vec2(i as f32 * margin, margin * 2.);
                let shapes = widgets::arrow_shape(
                    pos,
                    self.exercise_params.arrow_size as f32,
                    direction,
                    to_screen,
                    Color32::KHAKI,
                );
                ui.painter().add(shapes);
            }
        }
    }

    /// Generate random arrow directions and push to answer vec.
    fn add_arrows(&mut self) {
        for _ in 0..self.exercise_params.num_arrows {
            if let Some(direction) = vec![
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ]
            .choose(&mut rand::thread_rng())
            {
                self.answer.push(*direction);
            }
        }
    }

    /// Read arrow keys and register response.
    fn read_keypress(&mut self, ctx: &egui::Context) {
        let mut eval = |response: Direction| {
            self.response.push(response);
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

    /// Calculate % of correct inputs for one response.
    /// Example: 2 out of 4 arrows are correct: returns 0.5.
    fn evaluate_response(&self) -> f32 {
        let mut responses: Vec<f32> = vec![];
        for (answer, response) in zip(&self.answer, &self.response) {
            if answer == response {
                responses.push(1.0)
            } else {
                responses.push(0.0)
            }
        }
        let mut result = 0.0;
        for r in responses {
            result += r;
        }
        result = result / self.response.len() as f32;
        result
    }

    /// Keeps track of answer, response, result progression.
    fn progressor(&mut self, ctx: &egui::Context) {
        // Repaint regularly to update timers!
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // When the evaluation time is up or number of reps is reached, stop immediately.
        if self.evaluation.is_finished() {
            self.session_status = ExerciseStage::Finished;
        }

        match self.session_status {
            ExerciseStage::Challenge =>
            // Setup and display answer
            {
                if self.answer.len() == 0 {
                    self.timer.set(
                        Duration::try_milliseconds(self.exercise_params.answer_timeout)
                            .unwrap_or_default(),
                    );
                    self.add_arrows();
                }
                if self.timer.is_finished() {
                    self.session_status = ExerciseStage::Response;
                }
            }
            ExerciseStage::Response =>
            // Allow response input
            {
                self.read_keypress(ctx);
                // When complete response is given, progress
                if self.response.len() == self.answer.len() {
                    // store evaluation result
                    let score = self.evaluate_response();
                    self.evaluation.add_result(score);
                    // Set a two second timer to display result
                    self.timer.set(Duration::try_seconds(2).unwrap_or_default());
                    // Progress to result
                    self.session_status = ExerciseStage::Result;
                }
            }
            ExerciseStage::Result =>
            // display answer and response,
            {
                // Finally, restart
                if self.timer.is_finished() {
                    self.answer.clear();
                    self.response.clear();
                    self.session_status = ExerciseStage::Challenge;
                }
            }
            // None and Finished don't trigger progression.
            _ => (),
        };
    }
}

// ***********
// UI
// ***********
impl VisRecognition {
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

impl Exercise for VisRecognition {
    fn name(&self) -> &'static str {
        "Recognition"
    }

    fn description(&self) -> &'static str {
        "Remember what you have seen."
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    fn excercise_type(&self) -> Vec<ExerciseType> {
        vec![ExerciseType::Cognitive, ExerciseType::Visual]
    }

    fn help(&self) -> &'static str {
        "This exercise shows a number of arrows. After the arrows disappear, quickly enter the corresponding arrows on the keyboard."
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut tts::Tts) {
        let default_window = egui::Window::new(self.name())
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
            ExerciseStage::None => {
                default_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            // After an evaluation show the review
            ExerciseStage::Finished => {
                default_window.show(ctx, |ui| self.finished_screen(ui));
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
        // Show help
        ui.label(self.help());
        ui.separator();

        // Display the evaluation config
        eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 120],
            [30, 120],
        );

        // Anonymous function that uses the exercise config
        let mut func = |exercise: &VisRecognitionConfig| {
            self.exercise_params = exercise.to_owned();
            self.session_status = ExerciseStage::Challenge;
            self.evaluation.start();
        };

        // Display exercise configs
        if let Some(config) = &appdata.excconfig {
            if let Some(config) =
                exercise_config_menu::<VisRecognitionConfig>(ui, &config.visual_recognition, 2)
            {
                func(config)
            };
        }
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {
        self.ui_controls(ui);
        if self.session_status == ExerciseStage::Finished {
            ui.label("Score: ");
            for i in self.evaluation.show_results() {
                ui.label(format!("{}", i));
            }
        } else {
            Frame::dark_canvas(ui.style()).show(ui, |ui| self.arrow_painter(ui));
        }
    }
}
