use chrono::Duration;
use eframe::emath;
use eframe::epaint::PathShape;
use egui::style::Margin;
use egui::{pos2, vec2, Align, Frame, Key, Rect, Stroke, Vec2};
use tts::Tts;

use crate::asset_loader::AppData;
use crate::evaluation::Evaluation;
use crate::widgets;
use crate::widgets::evaluation::eval_config_widgets;
use crate::wm::sessionman::Exercise;

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
pub struct DepthPerception {
    anaglyph: Anaglyph,
    calibrating: bool,
    evaluation: Evaluation<bool>,
    session: Session,
    step: isize,
}

impl Default for DepthPerception {
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

    /// Shows a menu to calibrate the colors used in the anaglyph painting.
    /// Different glasses for viewing anaglyphs exist, user must be able to
    /// set colors for optimal effect.
    /// TODO: there is currently no option to permanently save calibration data.
    fn calibrate(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("Calibrate the colors for your anaglyph glasses so each color is clearly visible to one eye, but hardly visible to the other. When properly calibrated the two diamonds may appear as one when seen through the glasses.");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Left eye");
                ui.color_edit_button_srgba(&mut self.anaglyph.color.left);
                ui.add_space(ui.available_width() / 3.);

                ui.color_edit_button_srgba(&mut self.anaglyph.color.right);
                ui.label("Right eye");
            });

            ui.separator();

            Frame::dark_canvas(ui.style())
                .outer_margin(Margin::from(0.0))
                // TODO: look into eliminating visible margin
                // (negative number works but what are the downsides?)
                .show(ui, |ui| {
                    let space = ui.available_size();
                    let center = {
                        // Determine size of drawing surface
                        let (_id, rect) = ui.allocate_space(space);
                        // Create a transform mapping the available space on a rectangle
                        let to_screen = emath::RectTransform::from_to(
                            Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                            rect,
                        );
                        // the center is at half the x width
                        let center = pos2(0.5, 0.0);
                        to_screen * center
                    };

                    // diamond is hardcoded to be half the width of the frame
                    let diamond_size = space[0] / 2.;

                    // calculte the vertices of a diamond
                    let gen_points = |x_offset_fraction: f32| {
                        let x_offset = x_offset_fraction * diamond_size;
                        let mut array = vec![];
                        let diamond_points = [
                            vec2(0.0, 0.5 * diamond_size),          // left
                            vec2(0.5 * diamond_size, 0.),           // top
                            vec2(diamond_size, 0.5 * diamond_size), // right
                            vec2(0.5 * diamond_size, diamond_size), // bottom
                        ];
                        let mut offset = center.clone();
                        offset[0] += x_offset; // offset horizontally
                        offset[1] -= diamond_size / 2.; // center vertically
                        for item in diamond_points {
                            array.push(offset + item.clone());
                        }
                        array
                    };

                    let left_diamond = {
                        let points = gen_points(-0.8);
                        PathShape::convex_polygon(points, self.anaglyph.color.left, Stroke::NONE)
                    };
                    let right_diamond = {
                        let points = gen_points(-0.2);
                        PathShape::convex_polygon(points, self.anaglyph.color.right, Stroke::NONE)
                    };

                    ui.painter().add(left_diamond);
                    ui.painter().add(right_diamond);

                });

            ui.horizontal(|ui| {
                if ui.button("Swap").clicked() {
                    let tmp = self.anaglyph.color.left;
                    self.anaglyph.color.left = self.anaglyph.color.right;
                    self.anaglyph.color.right = tmp;
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    self.anaglyph = Anaglyph::default();
                    self.calibrating = false
                }
                if ui.button("Save and close").clicked() {
                    self.calibrating = false
                }
            });
        });
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
