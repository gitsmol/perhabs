use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Duration, Local};
use eframe::emath;
use eframe::epaint::PathShape;
use egui::style::Margin;
use egui::{pos2, vec2, Align, Align2, Frame, Key, Rect, Stroke};

use crate::modules::exercises::anaglyph::Anaglyph;
use crate::windowman::{AppWin, View};
use crate::configs::{read_config, Exercise};
use perhabs::Direction;

pub struct Session {
    pub active: bool,
    pub start_time: DateTime<Local>,
    pub duration: Duration,
    pub results: Vec<bool>,
    pub answer_thresh: bool,
    pub step: isize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            active: false,
            start_time: Local::now(),
            duration: Duration::seconds(0),
            results: vec![],
            answer_thresh: false,
            step: 0,
        }
    }
}

struct Configuration {
    calibrating: bool,
    exercises: Vec<Exercise>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self { 
            calibrating: false,
            // TODO centralize all such config paths
            exercises: {
                let config_file = PathBuf::from_str("appdata/exercise_configs.json").unwrap_or_default(); 
                let config = read_config(&config_file);
                config.exercises
                
            }
        }
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

impl AppWin for Vergence {
    fn name(&self) -> &'static str {
        "Vergence"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, _spk: &mut tts::Tts) {
        if open == &true {
            
            //determine center of screen to anchor startup window
            let center = {
                let mut center = Align2::CENTER_TOP;
                center[1] = Align::Center;
                center
            };

            if !self.session.active {
                egui::Window::new("Vergence")
                    .collapsible(false)
                    .resizable(false)
                    .fixed_size([250., 300.])
                    .anchor(center, vec2(0., 0.))
                    .show(ctx, |ui| {
                        self.menu(ui);
                    });
            }

            if self.session.active {
                egui::CentralPanel::default().show(ctx, |ui| self.ui(ui, _spk));
            }

            self.read_keypress(ctx);
        }
    }
}

impl View for Vergence {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = Session::default();
                self.anaglyph.reset();
            };
        // ui.add_space(ui.available_width());
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
                .outer_margin(Margin::from(0.0)) // TODO: look into eliminating visible margin
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
                if ui.button("Cancel").clicked() {
                    self.configuration.calibrating = false
                }
                if ui.button("Save and close").clicked() {
                    self.configuration.calibrating = false
                }
            });
        });
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        if self.configuration.calibrating {
            self.calibrate(ui);
            return;
        }
        
        ui.vertical(|ui| {
            for excercise in &self.configuration.exercises {
                ui.label(format!("{}", excercise.name));
                ui.horizontal(|ui| {
                    for level in &excercise.levels {
                        if ui.button(&level.name).clicked() {
                            self.session.step = level.params.step;
                            self.session.active = true;
                        }
                    };    
                });                
            };
            
            ui.allocate_space(egui::Vec2 { x: 0., y: 10. });
            if ui.button("Calibrate").clicked() {
                self.configuration.calibrating = true
            }
        });
        


        // Fill space
        ui.allocate_space(ui.available_size());
    }

    fn read_keypress(&mut self, ctx: &egui::Context) {
        let mut eval = |a: Direction| {
            // If the answer is correct, add true to the results vec.
            // If the previous answer was also correct (indicated by the answer threshold),
            // increase the difficulty of the exercise.
            // If the previous answer was not correct, set the answer threshold to true.
            if a == self.anaglyph.focal_position {
                self.session.results.push(true);
                match self.session.results.last() {
                    Some(prev_val) => {
                        if prev_val == &true && self.session.answer_thresh == true {
                            self.session.answer_thresh = false;
                            self.anaglyph.background_offset += self.session.step;
                        } if prev_val == &true && self.session.answer_thresh == false {
                            self.session.answer_thresh = true
                        } if prev_val == &false {
                            self.session.answer_thresh = false   
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
                match self.session.results.last() {
                    Some(prev_val) => {
                        if prev_val == &false && self.session.answer_thresh == true {
                            self.session.answer_thresh = false;
                            self.anaglyph.background_offset = 0;
                        } else {
                            self.session.answer_thresh = true
                        }
                    }
                    None => (),
                }
            }
            // create arrays for a new anaglyph
            self.anaglyph.initialize();
        };

        if ctx.input().key_pressed(Key::ArrowUp) {
            eval(Direction::Up)
        };
        if ctx.input().key_pressed(Key::ArrowDown) {
            eval(Direction::Down)
        };
        if ctx.input().key_pressed(Key::ArrowLeft) {
            eval(Direction::Left)
        };
        if ctx.input().key_pressed(Key::ArrowRight) {
            eval(Direction::Right)
        };
    }
}
