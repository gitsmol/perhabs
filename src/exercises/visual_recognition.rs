use std::iter::zip;

use crate::{
    modules::{
        asset_loader::{exercise_config::ExerciseConfig, AppData},
        evaluation::Evaluation,
        timer::Timer,
        widgets::{self, menu_button},
    },
    wm::sessionman::Exercise,
};
use chrono::Duration;
use egui::{
    emath::{self},
    pos2, vec2, Align, Color32, Frame, Key, Rect, Vec2,
};
use perhabs::Direction;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
enum SessionStatus {
    None,
    Answer,
    Response,
    Result,
    Finished,
}

/// Params for a visual recognition exercise.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VisRecognitionExercise {
    name: String,
    num_arrows: usize,
    arrow_size: usize,
    answer_timeout: i64, // The number of milliseconds the answer is shown
}

impl Default for VisRecognitionExercise {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            num_arrows: 3,
            arrow_size: 3,
            answer_timeout: 500, // The number of milliseconds the answer is shown
        }
    }
}

impl ExerciseConfig for VisRecognitionExercise {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Visual exercise to train quick recognition and retention of shapes.
/// Draws a number of arrows in the middle of the screen. The arrows remain visible
/// for a short period of time. Then player presses the arrow keys to indicate the
/// sequence of arrows they have seen.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VisRecognition {
    session_status: SessionStatus,
    exercise_params: VisRecognitionExercise,
    answer: Vec<Direction>,   // The correct answer: a sequence of directions
    timer: Timer,             // Keeps track of timeouts
    response: Vec<Direction>, // The response given by the player
    evaluation: Evaluation<f32>,
}

impl Default for VisRecognition {
    fn default() -> Self {
        Self {
            session_status: SessionStatus::None,
            exercise_params: VisRecognitionExercise::default(),
            answer: vec![],
            timer: Timer::new(),
            response: vec![],
            evaluation: Evaluation::new(Duration::seconds(10), 3),
        }
    }
}

impl VisRecognition {
    /// Basic controls during a session
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                // Reset the whole exercise.
                *self = VisRecognition::default();
            };
            ui.label(format!(
                "Time: {}",
                self.evaluation.time_remaining().to_string()
            ));
            ui.label(format!("Reps: {}", self.evaluation.reps_remaining()));
        });
    }

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
        if self.session_status == SessionStatus::Answer
            || self.session_status == SessionStatus::Result
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
        if self.session_status == SessionStatus::Response
            || self.session_status == SessionStatus::Result
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
        ctx.request_repaint_after(std::time::Duration::from_millis(50));

        // When the evaluation time is up or number of reps is reached, stop immediately.
        if self.evaluation.is_finished() {
            self.session_status = SessionStatus::Finished;
        }

        match self.session_status {
            SessionStatus::Answer =>
            // Setup and display answer
            {
                if self.answer.len() == 0 {
                    self.timer
                        .set(Duration::milliseconds(self.exercise_params.answer_timeout));
                    self.add_arrows();
                }
                if self.timer.is_finished() {
                    self.session_status = SessionStatus::Response;
                }
            }
            SessionStatus::Response =>
            // Allow response input
            {
                self.read_keypress(ctx);
                // When complete response is given, progress
                if self.response.len() == self.answer.len() {
                    // store evaluation result
                    let score = self.evaluate_response();
                    self.evaluation.add_result(score);
                    // Set a two second timer to display result
                    self.timer.set(Duration::seconds(2));
                    // Progress to result
                    self.session_status = SessionStatus::Result;
                }
            }
            SessionStatus::Result =>
            // display answer and response,
            {
                // Finally, restart
                if self.timer.is_finished() {
                    self.answer.clear();
                    self.response.clear();
                    self.session_status = SessionStatus::Answer;
                }
            }
            // None and Finished don't trigger progression.
            _ => (),
        };
    }

    /// Return the average of all scores.
    fn calculate_total_average_score(&self) -> f32 {
        // Calculate average score.
        let mut total_score = 0.0;
        for r in self.evaluation.show_results() {
            total_score += r;
        }
        total_score / self.evaluation.show_results().len() as f32
    }

    /// Review the evaluation.
    fn finished_screen(&mut self, ui: &mut egui::Ui) {
        // Format total average score.
        let total_score = self.calculate_total_average_score();
        let total_score_color = match total_score {
            x if x > 0.8 => Color32::GREEN,
            x if x > 0.5 => Color32::BLUE,
            _ => Color32::from_rgb(255, 165, 0),
        };
        let total_score_formatted = format!("{:.0}%", total_score * 100.0);

        ui.horizontal(|ui| {
            widgets::circle_with_data(
                ui,
                &self.evaluation.reps_done().to_string(),
                &String::from("Reps done"),
                100.,
                Color32::BLUE,
            );
            widgets::circle_with_data(
                ui,
                &self.evaluation.time_taken_as_string(),
                &String::from("Time taken"),
                100.,
                Color32::BLUE,
            );

            widgets::circle_with_data(
                ui,
                &total_score_formatted,
                &String::from("Average score"),
                100.,
                total_score_color,
            );
        });

        // Scroll through results
        egui::ScrollArea::new([true, true])
            .max_height(300.)
            .show(ui, |ui| {
                ui.collapsing("Results", |ui| {
                    for r in self.evaluation.show_results() {
                        ui.label(format!("{}", r));
                    }
                });
            });

        // Close
        if ui.button("Close").clicked() {
            *self = VisRecognition::default();
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
        *self = VisRecognition::default();
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
            SessionStatus::None => {
                default_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            // After an evaluation show the review
            SessionStatus::Finished => {
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
        let mut func = |exercise: &VisRecognitionExercise| {
            self.exercise_params = exercise.to_owned();
            self.session_status = SessionStatus::Answer;
            self.evaluation.start();
        };

        if let Some(config) = &appdata.excconfig {
            let buttons_total: f32 = config.visual_recognition.len() as f32;
            let col_1_range = buttons_total - (buttons_total / 2.).floor();

            ui.columns(2, |col| {
                // Column 1 gets populated with at least half the buttons
                for i in 0..col_1_range as usize {
                    if let Some(exercise) = config.visual_recognition.get(i) {
                        if menu_button(&mut col[0], exercise.name(), "").clicked() {
                            func(exercise);
                        };
                    };
                }

                // Column 2 gets populated with the remaining buttons
                for i in col_1_range as usize..buttons_total as usize {
                    if let Some(exercise) = config.visual_recognition.get(i) {
                        if menu_button(&mut col[1], exercise.name(), "").clicked() {
                            func(exercise);
                        };
                    };
                }
            });
        };
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {
        self.ui_controls(ui);
        if self.session_status == SessionStatus::Finished {
            ui.label("Score: ");
            for i in self.evaluation.show_results() {
                ui.label(format!("{}", i));
            }
        } else {
            Frame::dark_canvas(ui.style()).show(ui, |ui| self.arrow_painter(ui));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::exercises::visual_recognition::VisRecognition;

    #[test]
    fn calculate_total_score() {
        let mut exercise = VisRecognition::default();
        exercise.evaluation.add_result(0.3);
        exercise.evaluation.add_result(1.0);
        exercise.evaluation.add_result(0.5);
        let score = exercise.calculate_total_average_score();
        assert_eq!(score, 0.59999996);
    }
}
