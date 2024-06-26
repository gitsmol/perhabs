use crate::exercises::Direction;
use eframe::{emath, epaint::RectShape};
use egui::{pos2, style::Margin, vec2, Color32, Frame, Pos2, Rect, Shape};
use ndarray::Array2;
use ndarray_rand::{rand_distr::Binomial, RandomExt};
use rand::prelude::*;
use std::{error::Error, iter::zip};

use super::AnaglyphColor;

pub enum Eye {
    Left,
    Right,
}

struct AnaglyphArrays {
    background_left: Array2<u64>,
    background_right: Array2<u64>,
    focal: Array2<u64>,
    focal_mask: Array2<u64>,
    diamond: Vec<usize>,
}

impl Default for AnaglyphArrays {
    fn default() -> Self {
        Self {
            background_left: Array2::zeros((100, 100)),
            background_right: Array2::zeros((100, 100)),
            focal: Array2::zeros((10, 10)),
            focal_mask: Array2::zeros((10, 10)),
            diamond: vec![],
        }
    }
}

pub struct Debug {
    draw_left: bool,
    draw_right: bool,
    focal_mark: bool,
    size_info: String,
}
impl Default for Debug {
    fn default() -> Self {
        Self {
            draw_left: true,
            draw_right: true,
            focal_mark: false,
            size_info: String::new(),
        }
    }
}

/// Struct for anaglyph images. Each image consists of a background and a focal point.
/// draw() draws both focal point and background in order, according to the pub variables.
/// Focal offset creates the illusion of the focal point being in front of the background.
/// To achieve this effect, the focal point is shifted slightly to the center of vision
/// relative to the background. The brain interprets this as the object being closer.
pub struct Anaglyph {
    pub background_offset: isize,
    pub screen_offset: Option<Pos2>,
    pub pixel_size: isize,
    pub grid_size: usize,
    focal_offset: isize, // The offset creates the illusion of depth
    pub focal_size_rel: f32,
    pub focal_position: Direction, // Where is the focal point?
    pub color: AnaglyphColor,
    arrays: AnaglyphArrays,
    pub debug: Debug,
}

impl Default for Anaglyph {
    fn default() -> Self {
        Self {
            screen_offset: None,
            pixel_size: 3,
            grid_size: 100,
            arrays: AnaglyphArrays::default(),
            background_offset: 0,
            focal_offset: 2,
            focal_size_rel: 0.35,
            focal_position: Direction::Up,
            color: AnaglyphColor::default(),
            debug: Debug::default(),
        }
    }
}

impl Anaglyph {
    /// - Generate random arrays of 1's and 0's for left and right backgrounds.
    /// - Calculate a diamond shape for the focal glyphs.
    /// - Remove the 'background' to the focal glyphs to create depth illusion (occlusion).
    pub fn initialize(&mut self) {
        // Create necessary arrays
        let distr = Binomial::new(1, 0.5).unwrap();
        self.arrays.background_left = Array2::random((self.grid_size, self.grid_size), distr);
        self.arrays.background_right = self.arrays.background_left.clone();
        self.arrays.focal = Array2::random((self.grid_size, self.grid_size), distr);
        self.arrays.focal_mask = Array2::zeros((self.grid_size, self.grid_size));
        let mut rng = thread_rng();
        self.focal_position = match rng.gen_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Left,
            2 => Direction::Right,
            3 => Direction::Down,
            _ => panic!("Fatal error in focal position random number generator."),
        };

        let focal_size = (self.grid_size as f32 * self.focal_size_rel) as usize;

        // create a matrix containing the diamond shape focal point
        let mut diamond = vec![];
        // step means: how many extra pixels to draw on each subsequent row of the matrix
        // by incrementing the amount of pixels to draw, a triangle shape is created
        let step = (focal_size / (focal_size / 2)) as usize;
        // create a vec of the number of pixels to draw on each row
        for i in (0..focal_size).step_by(step) {
            diamond.push(i)
        }
        // finally, extend the vec with itself reversed,
        // effectively appending an upside down triangle
        let mut rev = diamond.clone();
        rev.reverse();
        diamond.extend(rev);

        self.arrays.diamond = diamond;

        // Return a tuple giving the position of the focal point relative to the background
        let focal_loc = match self.focal_position {
            Direction::Up => (0.5, 0.25),
            Direction::Down => (0.5, 0.75),
            Direction::Left => (0.25, 0.5),
            Direction::Right => (0.75, 0.5),
        };

        // Determine the starting point for 'drawing' the diamond shape in the array
        let focal_size = self.grid_size as f32 * self.focal_size_rel;
        let x_min = self.grid_size as f32 * focal_loc.0;
        let y_min = self.grid_size as f32 * focal_loc.1 - focal_size / 2.;
        let (x_min, y_min) = (x_min as usize, y_min as usize);

        let diamond_zip = zip(
            0..self.arrays.diamond.len(), // create a zip of rowcount
            self.arrays.diamond.iter(),   // and number of pixels
        );

        // A row has a row number and a number of pixels to fill in
        for (row, pixels) in diamond_zip {
            // Determine x and y coords for the topleft of the first pixel in this row
            let y = y_min + row; // add the current row to the topmost coord of the glyph
            let x = x_min - pixels / 2; // Find the horizontal starting coord considering
                                        // half the pixels must be drawn left of center

            // Iterate over the number of pixels in this row
            for x in x..(x + pixels) {
                // self.arrays.focal_mask[[x + self.focal_offset as usize, y]] = 1;
                self.arrays.focal_mask[[x, y]] = 1;
                // remove the diamond from the bg array (left)
                self.arrays.background_left[[x + self.focal_offset as usize, y]] = 0;
                // remove the diamond from the bg array (right)
                self.arrays.background_right[[x - self.focal_offset as usize, y]] = 0;
            }
        }
    }

    /// Generates shapes to be drawn.
    /// The focal point is drawn according to the shape in the focal_mask array.
    fn draw_pixels(self: &mut Self, eye: Eye, origin: &Pos2) -> Result<Vec<Shape>, Box<dyn Error>> {
        // Left/right image gets appropriate coloring and the offset value is split between them
        let (color, bg_offset, focal_offset, background) = match eye {
            Eye::Left => (
                self.color.left,
                self.background_offset * -1, // move bg to the left
                self.focal_offset,
                &self.arrays.background_left,
            ),
            Eye::Right => (
                self.color.right,
                self.background_offset,
                self.focal_offset * -1, // move focal to the left
                &self.arrays.background_right,
            ),
        };
        let pixel_size = &self.pixel_size;

        // Create rectangles ('pixels') and push them to a vec
        let mut rects = vec![];
        for y in 0..self.grid_size {
            // fill in each row
            for x in 0..self.grid_size {
                // only create a 'pixel' if the random seed is 1 for this coord
                if background[[x, y]] == 1 {
                    // pixel starts at row number * pixel size + or - the background offset
                    let coords_min = vec2(
                        (x as isize * pixel_size + bg_offset) as f32,
                        (y as isize * pixel_size) as f32,
                    );
                    let coords_max = coords_min + vec2(*pixel_size as f32, *pixel_size as f32);
                    let pixel = RectShape::filled(
                        Rect {
                            min: *origin + coords_min,
                            max: *origin + coords_max,
                        },
                        0.0,
                        color,
                    );
                    rects.push(Shape::Rect(pixel));
                }

                let Some(focal_mask_coord) = self.arrays.focal_mask.get([x, y]) else {
                    return Err("Couldn't index into focal mask array.".into());
                };
                let Some(focal_coord) = self.arrays.focal.get([x, y]) else {
                    return Err("Couldn't index into focal mask array.".into());
                };

                if focal_mask_coord == &1 && focal_coord == &1 {
                    // pixel starts at (left top)
                    let coords_min = vec2(
                        (x as isize * pixel_size + focal_offset * pixel_size + bg_offset) as f32,
                        (y as isize * pixel_size) as f32,
                    );
                    // pixel ends at (right bottom)
                    let coords_max = coords_min + vec2(*pixel_size as f32, *pixel_size as f32);
                    let pixel = RectShape::filled(
                        Rect {
                            min: *origin + coords_min,
                            max: *origin + coords_max,
                        },
                        0.0,
                        if self.debug.focal_mark == true {
                            Color32::from_additive_luminance(192)
                        } else {
                            color
                        },
                    );
                    rects.push(Shape::Rect(pixel));
                }
            }
        }
        Ok(rects)
    }

    /// Draw the background pixels and the focal pixes for left and right eye.
    pub fn draw(self: &mut Self, ui: &mut egui::Ui) -> Result<(), Box<dyn Error>> {
        Frame::dark_canvas(ui.style())
            .outer_margin(Margin::from(0.0))
            .show(ui, |ui| -> Result<(), Box<dyn Error>> {
                let origin = {
                    // Determine size of drawing surface: full screen
                    let available_size = ui.available_size_before_wrap();
                    let (_id, rect) = ui.allocate_space(available_size);
                    // Determine starting coords to end up with a centered drawing
                    let to_screen = emath::RectTransform::from_to(
                        Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), // 0.5 is center
                        rect,
                    );

                    // We need to determine where on the screen to draw the glyph.
                    // If we have a screen_offset in the analgyph struct, that determines
                    // where we draw the glyph. If we don't, we center the glyph.
                    let screen_offset = match self.screen_offset {
                        Some(screen_offset) => screen_offset,
                        None => {
                            // how many pixels do we need?
                            let drawsize: f32 = self.grid_size as f32 * self.pixel_size as f32;
                            let rel_size_x = drawsize / available_size[0] / 2.; // how wide is half a drawing?
                            let rel_size_y = drawsize / available_size[1] / 2.; // how high is half a drawing?

                            let screen_offset = pos2(0.5 - rel_size_x, 0.5 - rel_size_y);

                            // debug info
                            self.debug.size_info = format!(
                                "desired_size: {:?} | rel_size_y: {}",
                                available_size, rel_size_y
                            );

                            screen_offset
                        }
                    };

                    to_screen * screen_offset
                };

                if self.debug.draw_left == true {
                    let left = self.draw_pixels(Eye::Left, &origin)?; // TODO error handling
                    ui.painter().extend(left);
                };
                if self.debug.draw_right == true {
                    let right = self.draw_pixels(Eye::Right, &origin)?; // TODO error handling
                    ui.painter().extend(right);
                };
                Ok(())
            });
        Ok(())
    }

    /// Show some simple debugging controls
    pub fn debug_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::Checkbox::new(&mut self.debug.draw_left, "Left"));
            ui.add(egui::Checkbox::new(&mut self.debug.draw_right, "Right"));
            ui.add(egui::Checkbox::new(
                &mut self.debug.focal_mark,
                "Focal mark",
            ));
            ui.label(&self.debug.size_info);
        });
        ui.horizontal(|ui| {
            if ui
                .add(egui::Slider::new(&mut self.pixel_size, 1..=10).suffix("pixel size"))
                .changed()
            {
                self.initialize()
            };
            if ui
                .add(egui::Slider::new(&mut self.grid_size, 10..=150).suffix("anaglyph size"))
                .changed()
            {
                self.initialize()
            };
            if ui
                .add(
                    egui::Slider::new(&mut self.background_offset, -30..=30)
                        .suffix("bg_offset size"),
                )
                .changed()
            {
                self.initialize()
            };
            if ui
                .add(egui::Slider::new(&mut self.focal_offset, 0..=10).suffix("focal_offset"))
                .changed()
            {
                self.initialize()
            };
        });
    }

    /// What is the normalized size of the anaglyph?
    pub fn size(&self) -> usize {
        self.grid_size * self.pixel_size.abs() as usize
    }
}
