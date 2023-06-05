use chrono::{DateTime, Duration, Local};
use eframe::emath;
use eframe::epaint::PathShape;
use egui::style::Margin;
use egui::{pos2, vec2, Align, Frame, Key, Rect, Stroke, Vec2};
use tts::Tts;

use crate::modules::asset_loader::AppData;
use crate::wm::sessionman::Exercise;

use perhabs::Direction;

use self::anaglyph::Anaglyph;

mod anaglyph;

pub struct Session {
    pub active: bool,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub results: Vec<bool>,
    pub answer_thresh_success: bool,
    pub answer_thresh_fail: bool,
    pub step: isize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            active: false,
            start_time: Local::now(),
            duration: Duration::seconds(0),
            results: vec![],
            answer_thresh_success: false,
            answer_thresh_fail: false,
            step: 0,
        }
    }
}

struct Configuration {
    calibrating: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self { calibrating: false }
    }
}
/// Exercise to train binocular convergence/divergence usign anaglyph images.
pub struct Vergence {
    anaglyph: Anaglyph,
    session: Session,
    configuration: Configuration,
}

impl Default for Vergence {
    fn default() -> Self {
        Self {
            anaglyph: Anaglyph::default(),
            session: Session::default(),
            configuration: Configuration::default(),
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
        *self = Vergence::default();
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        if !self.session.active {
            egui::Window::new("Vergence")
                .anchor(
                    egui::Align2([Align::Center, Align::TOP]),
                    Vec2::new(0., 100.),
                )
                .fixed_size(vec2(350., 300.))
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    self.ui(ui, appdata, tts);
                });
        }

        if self.session.active {
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }

        self.read_keypress(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        if self.configuration.calibrating {
            self.calibrate(ui);
            return;
        }

        ui.label("This excercise shows a square. Inside the square is a diamond. Press the arrow key to indicate where you see the diamond in the square: left, right, up or down.");
        ui.separator();

        egui::Grid::new("vergence_selector_grid")
            .num_columns(4)
            .show(ui, |ui| {
                if let Some(excconfig) = &appdata.excconfig {
                    for excercise in &excconfig.vergence {
                        ui.label(format!("{}", excercise.name));

                        for level in &excercise.levels {
                            if ui.button(&level.name).clicked() {
                                self.session.step = level.step;
                                self.anaglyph.pixel_size = level.pixel_size;
                                self.session.active = true;
                            }
                        }

                        ui.end_row();
                    }
                }
            });

        // Fill space
        ui.allocate_space(ui.available_size());

        if ui.button("Calibrate").clicked() {
            self.configuration.calibrating = true
        }
    }
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = Session::default();
                self.anaglyph.reset();
            };
            ui.checkbox(&mut self.anaglyph.debug.show, "Debug");
        });

        self.anaglyph.draw(ui);
    }
}

impl Vergence {
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
                    self.configuration.calibrating = false
                }
                if ui.button("Save and close").clicked() {
                    self.configuration.calibrating = false
                }
            });
        });
    }

    fn read_keypress(&mut self, ctx: &egui::Context) {
        let mut eval = |a: Direction| {
            // If the answer is correct, add true to the results vec.
            // If the previous answer was also correct (indicated by the answer threshold),
            // increase the difficulty of the exercise.
            // If the previous answer was not correct, set the answer threshold to true.
            if a == self.anaglyph.focal_position {
                self.session.results.push(true);
                // Any correct answer invalidates the failure streak.
                self.session.answer_thresh_fail = false;
                match self.session.results.last() {
                    Some(prev_val) => {
                        if prev_val == &true && self.session.answer_thresh_success == true {
                            self.session.answer_thresh_success = false;
                            self.anaglyph.background_offset += self.session.step;
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
                self.session.results.push(false);
                // Any failure invalidates the success streak.
                self.session.answer_thresh_success = false;
                match self.session.results.last() {
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
