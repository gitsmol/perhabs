use crate::exercises::Direction;
use crate::shared::asset_loader::exercise_config::visual_saccades::VisSaccadesConfig;
use crate::shared::AppData;
use crate::widgets::evaluation::eval_config_widgets;
use crate::widgets::exercise_config_menu::exercise_config_menu_multicol;
use crate::widgets::{self};
use crate::{
    wm::Exercise,
    {shared::Evaluation, shared::Timer},
};
use chrono::Duration;
use egui::{emath, pos2, vec2, Align, Color32, Frame, Key, Pos2, Rect, Sense, Vec2};
use rand::{seq::SliceRandom, Rng};

use super::ExerciseStatus;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VisSaccades {
    session_status: ExerciseStatus,
    arrow_pos: Option<Pos2>,
    answer: Option<Direction>, // The right answer is the direction of the arrow
    response: Option<Direction>, // The given response is a direction
    exercise_params: VisSaccadesConfig,
    answer_timeout_timer: Timer,
    evaluation: Evaluation<f32>,
}

impl Default for VisSaccades {
    fn default() -> Self {
        Self {
            session_status: ExerciseStatus::None,
            exercise_params: VisSaccadesConfig::default(),
            arrow_pos: None,
            answer: None,
            response: None,
            answer_timeout_timer: Timer::new(),
            evaluation: Evaluation::new(Duration::seconds(60), 60),
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl VisSaccades {
    fn arrow_painter(&self, ui: &mut egui::Ui) {
        // Set up
        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            Sense::focusable_noninteractive(),
        );
        let to_screen = emath::RectTransform::from_to(
            Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0),
            response.rect,
        );

        if let Some(pos) = self.arrow_pos {
            if let Some(direction) = &self.answer {
                let shape = widgets::arrow_shape(
                    pos,
                    self.exercise_params.arrow_size as f32,
                    direction,
                    to_screen,
                    Color32::GREEN,
                );
                painter.add(shape);
            }
        }
    }

    /// Randomly position an arrow pointing in a random direction.
    fn new_arrow_pos(&mut self) {
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
        }

        let x: f32 = rng.gen_range(0.01..0.95);
        let y: f32 = rng.gen_range(0.01..0.95);
        self.arrow_pos = Some(pos2(x, y));
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
                // If no arrow is visible, create new arrow and set answer timeout timer
                if let None = self.answer {
                    self.new_arrow_pos();
                    self.answer_timeout_timer
                        .set(Duration::milliseconds(self.exercise_params.answer_timeout));
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
impl VisSaccades {
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

impl Exercise for VisSaccades {
    fn name(&self) -> &'static str {
        "Scanning (Saccades)"
    }

    fn description(&self) -> &'static str {
        "Quickly scan the screen and respond."
    }

    fn reset(&mut self) {
        *self = VisSaccades::default();
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

        // Display all exercise configs
        let mut func = |exercise: &VisSaccadesConfig| {
            self.exercise_params = exercise.to_owned();
            self.session_status = ExerciseStatus::Response;
            self.evaluation.start();
        };

        // Display exercise configs
        if let Some(config) = &appdata.excconfig {
            if let Some(config) =
                exercise_config_menu_multicol::<VisSaccadesConfig>(ui, &config.visual_saccades, 3)
            {
                func(config)
            };
        }
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {
        self.ui_controls(ui);
        Frame::dark_canvas(ui.style()).show(ui, |ui| self.arrow_painter(ui));
    }
}
