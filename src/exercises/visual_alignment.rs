use chrono::Duration;

use egui::{emath, pos2, vec2, Align, Key, Rect, Response, Rounding, Sense, Stroke, Vec2};
use egui::{Frame, Pos2};
use rand::Rng;
use tts::Tts;

use crate::shared::AnaglyphColor;
use crate::shared::AppData;
use crate::shared::Evaluation;
use crate::widgets;
use crate::wm::Exercise;

use super::ExerciseStatus;

/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct VisualAlignment {
    colors: AnaglyphColor,
    target_pos_normalized: Pos2,
    target_pos_on_screen: Pos2,
    sight_pos_on_screen: Pos2,
    calibrating: bool,
    evaluation: Evaluation<Vec2>,
    status: ExerciseStatus,
}

impl Default for VisualAlignment {
    fn default() -> Self {
        Self {
            colors: AnaglyphColor::default(),
            target_pos_normalized: Pos2::new(0.5, 0.5),
            target_pos_on_screen: Pos2::new(0.5, 0.5),
            sight_pos_on_screen: Pos2::new(100., 100.),
            calibrating: false,
            evaluation: Evaluation::new(Duration::seconds(60), 10),
            status: ExerciseStatus::None,
        }
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl VisualAlignment {
    /// Evaluate given answer
    fn evaluate_answer(&mut self) -> Vec2 {
        self.sight_pos_on_screen - self.target_pos_on_screen
    }

    fn read_keypress(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            self.sight_pos_on_screen.x += 1.0;
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            self.sight_pos_on_screen.y += 1.0;
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            self.sight_pos_on_screen.x -= 1.0;
        };
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.sight_pos_on_screen.y -= 1.0;
        };

        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            self.status = ExerciseStatus::Response;
        };
    }

    /// Keeps track of answer, response, result progression.
    fn progressor(&mut self, ctx: &egui::Context) {
        // Repaint regularly to update timers!
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // read keypresses
        self.read_keypress(ctx);

        // Stop when finished
        if self.evaluation.is_finished() {
            self.status = ExerciseStatus::Finished;
            return;
        };

        // When the user gives a response, process the response and
        // start the next challenge
        if self.status == ExerciseStatus::Response {
            let target_sight_offset = self.evaluate_answer();
            self.evaluation.add_result(target_sight_offset);
            self.target_pos_normalized = self.gen_random_pos2();
            self.status = ExerciseStatus::Challenge
        };
    }

    /// Paints the arrow shapes
    fn paint_target(&mut self, ui: &mut egui::Ui) -> Response {
        // Determine size of drawing surface and aspect ratio
        let (_id, rect) = ui.allocate_space(ui.available_size_before_wrap());
        let response = ui.allocate_rect(rect, Sense::click());

        // Create a transform mapping the available space on a rectangle,
        // taking aspect ratio into account
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);

        // Determine where to start drawing.
        let target_size = 30.;
        let stroke_size = 5.;
        self.target_pos_on_screen = to_screen * self.target_pos_normalized;

        // Paint the target
        ui.painter().rect_stroke(
            Rect::from_center_size(
                self.target_pos_on_screen,
                Vec2::new(target_size, target_size),
            ),
            Rounding::ZERO,
            Stroke::new(stroke_size, self.colors.left),
        );

        // Paint the 'sight' used to aim
        ui.painter().circle_stroke(
            self.sight_pos_on_screen,
            target_size / 2. - stroke_size,
            Stroke::new(stroke_size, self.colors.right),
        );

        response
    }

    /// Paints the arrow shapes
    fn paint_result_average_targets(&mut self, ui: &mut egui::Ui, response_diff: Vec2) -> Response {
        // Determine size of drawing surface and aspect ratio
        let (_id, rect) = ui.allocate_space(vec2(250., 200.));
        let response = ui.allocate_rect(rect, Sense::click());

        // Create a transform mapping the available space on a rectangle,
        // taking aspect ratio into account
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);

        // Determine where to start drawing.
        let target_size = 30.;
        let stroke_size = 5.;
        let target_pos_on_screen = to_screen * pos2(0.5, 0.5);
        let response_pos_on_screen = to_screen * pos2(0.5, 0.5) + response_diff;

        // Paint the target
        ui.painter().rect_stroke(
            Rect::from_center_size(target_pos_on_screen, Vec2::new(target_size, target_size)),
            Rounding::ZERO,
            Stroke::new(stroke_size, self.colors.left),
        );

        // Paint the 'sight' used to aim
        ui.painter().circle_stroke(
            response_pos_on_screen,
            target_size / 2. - stroke_size,
            Stroke::new(stroke_size, self.colors.right),
        );

        response
    }

    /// Generate random  directions and push to answer vec.
    fn gen_random_pos2(&mut self) -> Pos2 {
        let mut rng = rand::thread_rng();
        let x: f32 = rng.gen_range(0.2..0.8);
        let y: f32 = rng.gen_range(0.2..0.8);
        pos2(x, y)
    }
}

// ***********
// UI
// ***********
impl VisualAlignment {
    fn debug_ui(&self, ui: &mut egui::Ui) {
        let diff = self.target_pos_on_screen - self.sight_pos_on_screen;
        ui.horizontal(|ui| ui.label(format!("X diff: {}", diff.x)));
        ui.horizontal(|ui| ui.label(format!("Y diff: {}", diff.y)));
    }

    /// Basic controls during a session
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                // Reset the whole exercise.
                *self = Default::default();
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

        let x_shift = {
            let mut x_total = 0.;
            for result in self.evaluation.show_results() {
                x_total += result.x;
            }
            x_total / self.evaluation.show_results().len() as f32
        };

        let y_shift = {
            let mut total = 0.;
            for result in self.evaluation.show_results() {
                total += result.y;
            }
            total / self.evaluation.show_results().len() as f32
        };

        ui.horizontal(|ui| {
            ui.label("Average horizontal shift:");
            ui.label(format!("{:.2}", x_shift));
            if x_shift >= 0.0 {
                ui.label("to the right.")
            } else {
                ui.label("to the left.")
            }
        });
        ui.horizontal(|ui| {
            ui.label("Average vertical shift:");
            ui.label(format!("{:.2}", y_shift));
            if y_shift >= 0.0 {
                ui.label("down.")
            } else {
                ui.label("up.")
            }
        });

        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            self.paint_result_average_targets(ui, vec2(x_shift, y_shift));
        });

        // Close
        if ui.button("Close").clicked() {
            *self = Default::default();
        };
    }
}

impl Exercise for VisualAlignment {
    fn name(&self) -> &'static str {
        "Visual Alignment"
    }

    fn description(&self) -> &'static str {
        "Test the alignment of your eyes."
    }

    fn reset(&mut self) {
        // Remember color calibrations
        let tmp_color = self.colors.clone();
        *self = Default::default();
        self.colors = tmp_color;
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
        // - Active session
        // - No session shows the exercise menu
        match self.status {
            // Show menu
            ExerciseStatus::None => {
                menu_window.show(ctx, |ui| {
                    self.ui(ui, appdata, tts);
                });
            }

            // Show finished menu
            ExerciseStatus::Finished => {
                menu_window.show(ctx, |ui| self.finished_screen(ui));
            }

            // All other statuses mean we are in session
            _ => {
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
                // Track mouse position
                self.sight_pos_on_screen = ctx.input(|i| {
                    if let Some(position) = i.pointer.hover_pos() {
                        position + vec2(-50., -50.)
                    } else {
                        self.sight_pos_on_screen
                    }
                });
                self.progressor(ctx);
            }
        };
    }

    /// The exercise menu
    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        // Calibration guard clause
        if self.calibrating {
            widgets::calibrate_anaglyph::calibrate(ui, &mut self.colors, &mut self.calibrating);
            return;
        }

        ui.label("This excercise shows a square. Inside the square is a diamond. Press the arrow key to indicate where you see the diamond in the square: left, right, up or down.");
        ui.separator();

        widgets::evaluation::eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 120],
            [5, 30],
        );

        if ui.button("Start").clicked() {
            self.evaluation.start();
            self.status = ExerciseStatus::Challenge;
        }

        // Add some space and show calibration button
        ui.add_space(20.);

        if ui.button("Calibrate").clicked() {
            self.calibrating = true
        }
    }

    /// The session window showing anaglyphs
    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        if appdata.debug {
            self.debug_ui(ui);
        }
        // Ui controls
        self.ui_controls(ui);

        // Show target and sight on dark background
        Frame::dark_canvas(ui.style()).show(ui, |ui| {
            if self.paint_target(ui).clicked() {
                self.status = ExerciseStatus::Response;
            }
        });
    }
}
