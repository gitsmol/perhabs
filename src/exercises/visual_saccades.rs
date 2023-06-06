use crate::{
    modules::{
        asset_loader::{
            exercise_config::{visual_saccades::VisSaccadesExercise, ExerciseConfig},
            AppData,
        },
        evaluation::Evaluation,
        timer::Timer,
        widgets::{self, menu_button},
    },
    wm::sessionman::Exercise,
};
use chrono::Duration;
use egui::{emath, pos2, vec2, Align, Color32, Frame, Key, Pos2, Rect, Sense, Vec2};
use perhabs::Direction;
use rand::{seq::SliceRandom, Rng};

use super::SessionStatus;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VisSaccades {
    session_status: SessionStatus,
    arrow_pos: Option<Pos2>,
    answer: Option<Direction>, // The right answer is the direction of the arrow
    response: Option<Direction>, // The given response is a direction
    exercise_params: VisSaccadesExercise,
    answer_timeout_timer: Timer,
    evaluation: Evaluation<f32>,
}

impl Default for VisSaccades {
    fn default() -> Self {
        Self {
            session_status: SessionStatus::None,
            exercise_params: VisSaccadesExercise::default(),
            arrow_pos: None,
            answer: None,
            response: None,
            answer_timeout_timer: Timer::new(),
            evaluation: Evaluation::new(Duration::seconds(5), 5),
        }
    }
}

impl VisSaccades {
    /// Basic controls during a session
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                // Reset the whole exercise.
                *self = VisSaccades::default();
            };
            ui.label(format!(
                "Time: {}",
                self.evaluation.time_remaining().to_string()
            ));
            ui.label(format!("Reps: {}", self.evaluation.reps_remaining()));
        });
    }

    fn arrow_painter(&self, ui: &mut egui::Ui) {
        // Set up
        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            Sense::focusable_noninteractive(),
        );
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
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
        // 60 fps = 100ms per frame
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // When the evaluation time is up or number of reps is reached, stop immediately.
        if self.evaluation.is_finished() {
            self.session_status = SessionStatus::Finished;
        }

        match self.session_status {
            // This exercise is always in response mode.
            SessionStatus::Response => {
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
            if let Some(average_secs) = &self.evaluation.average_secs_per_rep() {
                widgets::circle_with_data(
                    ui,
                    &format!("{}s", average_secs),
                    &String::from("Avg response"),
                    100.,
                    Color32::BLUE,
                );
            }
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
            *self = VisSaccades::default();
        }
    }
}

impl Exercise for VisSaccades {
    fn name(&self) -> &'static str {
        "Tracking (Saccades)"
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
            SessionStatus::None => {
                menu_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            // After an evaluation show the review
            SessionStatus::Finished => {
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

    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut tts::Tts) {
        let mut func = |exercise: &VisSaccadesExercise| {
            self.exercise_params = exercise.to_owned();
            self.session_status = SessionStatus::Response;
            self.evaluation.start();
        };

        if let Some(config) = &appdata.excconfig {
            let buttons_total: f32 = config.visual_saccades.len() as f32;
            let col_1_range = buttons_total - (buttons_total / 2.).floor();

            ui.columns(2, |col| {
                // Column 1 gets populated with at least half the buttons
                for i in 0..col_1_range as usize {
                    if let Some(exercise) = config.visual_saccades.get(i) {
                        if menu_button(&mut col[0], exercise.name(), "").clicked() {
                            func(exercise);
                        };
                    };
                }

                // Column 2 gets populated with the remaining buttons
                for i in col_1_range as usize..buttons_total as usize {
                    if let Some(exercise) = config.visual_saccades.get(i) {
                        if menu_button(&mut col[1], exercise.name(), "").clicked() {
                            func(exercise);
                        };
                    };
                }
            });
        };
    }

    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut tts::Tts) {
        self.ui_controls(ui);
        Frame::dark_canvas(ui.style()).show(ui, |ui| self.arrow_painter(ui));
    }
}
