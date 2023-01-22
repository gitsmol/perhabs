use eframe::{emath, epaint::RectShape};
use egui::{pos2, style::Margin, vec2, Color32, Frame, Pos2, Rect, Shape};
use ndarray::Array2;
use ndarray_rand::{rand_distr::Binomial, RandomExt};
use perhabs::{Direction, PhError};
use std::iter::zip;

enum Eye {
    Left,
    Right,
}

pub struct AnaglyphColor {
    left: Color32,
    right: Color32,
}

impl Default for AnaglyphColor {
    fn default() -> Self {
        Self {
            left: Color32::from_rgba_unmultiplied(0, 38, 230, 100), // blue
            right: Color32::from_rgba_unmultiplied(255, 25, 25, 50), // red
        }
    }
}

struct AnaglyphArrays {
    background: Array2<u64>,
    background_left: Array2<u64>,
    background_right: Array2<u64>,
    focal: Array2<u64>,
    focal_mask: Array2<u64>,
    diamond: Vec<usize>,
}

impl Default for AnaglyphArrays {
    fn default() -> Self {
        Self {
            background: Array2::zeros((10, 10)),
            background_left: Array2::zeros((10, 10)),
            background_right: Array2::zeros((10, 10)),
            focal: Array2::zeros((10, 10)),
            focal_mask: Array2::zeros((10, 10)),
            diamond: vec![],
        }
    }
}

pub struct Debug {
    pub draw_left: bool,
    pub draw_right: bool,
    pub focal_mark: bool,
}

/// Struct for anaglyph images. Each image consists of a background and a focal point.
/// draw() draws both focal point and background in order, according to the pub variables.
pub struct Anaglyph {
    pub background_offset: isize,
    pub pixel_size: isize,
    pub grid_size: usize,
    pub focal_offset: isize, // The offset creates the illusion of depth
    pub focal_size_rel: f32,
    pub focal_position: Direction, // Where is the focal point?
    pub color: AnaglyphColor,
    arrays: AnaglyphArrays,
    pub debug: Debug,
}

impl Default for Anaglyph {
    fn default() -> Self {
        Self {
            pixel_size: 3,
            grid_size: 100,
            arrays: AnaglyphArrays::default(),
            background_offset: 0,
            focal_offset: 2,
            focal_size_rel: 0.35,
            focal_position: Direction::Up,
            color: AnaglyphColor::default(),
            debug: Debug {
                draw_left: true,
                draw_right: false,
                focal_mark: false,
            },
        }
    }
}

impl Anaglyph {
    /// Generate a random array of 1's and 0's. Then remove the 'background'
    /// to the focal glyphs to create depth illusion.
    pub fn gen_pixel_arrays(&mut self) {
        let distr = Binomial::new(1, 0.5).unwrap();
        self.arrays.background = Array2::random((self.grid_size, self.grid_size), distr);
        self.arrays.background_left = Array2::random((self.grid_size, self.grid_size), distr);
        self.arrays.background_right = self.arrays.background_left.clone();
        self.arrays.focal = Array2::random((self.grid_size, self.grid_size), distr);
        self.arrays.focal_mask = Array2::zeros((self.grid_size, self.grid_size));
        self.remove_focal_from_bg();
    }

    /// Calculates a diamond shape. Removes the datapoints for the focal point from
    /// the pixel_array used for the background. This creates the illusion of the focal point
    /// obscuring its background.
    fn remove_focal_from_bg(self: &mut Self) {
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

    /// Draws the background from a randomly generated self.bg_pixel_array.
    /// Draw_focal needs to be called beforehand so the pixels in the focal
    /// point are removed from the background array.
    ///
    /// Draw the focal point in the shape of a diamond in exactly the spot we removed from the background.
    /// Focal offset creates the illusion of the focal point being in front of the background.
    /// To achieve this effect, the focal point is shifted slightly to the center of vision
    /// relative to the background. The brain interprets this as the object being closer.
    fn draw_background(self: &mut Self, eye: Eye, origin: &Pos2) -> Result<Vec<Shape>, PhError> {
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
        let pixel_size = self.pixel_size;

        // Create rectangles and push them to a vec
        let mut rects = vec![];
        // create rows
        for y in 0..self.grid_size {
            // fill in each row
            for x in 0..self.grid_size {
                // only create a 'pixel' if the random seed is 1 for this coord
                // if self.arrays.background[[x, y]] == 1 {
                if background[[x, y]] == 1 {
                    let coords_min = vec2(
                        (x as isize * pixel_size + bg_offset) as f32,
                        (y as isize * pixel_size) as f32,
                    );
                    let coords_max = coords_min + vec2(pixel_size as f32, pixel_size as f32);
                    let sq = RectShape::filled(
                        Rect {
                            min: *origin + coords_min,
                            max: *origin + coords_max,
                        },
                        0.0,
                        color,
                    );
                    rects.push(Shape::Rect(sq));
                }
                if self.arrays.focal_mask[[x, y]] == 1 && self.arrays.focal[[x, y]] == 1 {
                    // draw focal pixels
                    let coords_min = vec2(
                        // (x as isize * pixel_size + bg_offset + focal_offset as isize) as f32,
                        (x as isize * pixel_size + bg_offset + focal_offset * pixel_size) as f32,
                        (y as isize * pixel_size) as f32,
                    );
                    let coords_max = coords_min + vec2(pixel_size as f32, pixel_size as f32);
                    let sq = RectShape::filled(
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
                    rects.push(Shape::Rect(sq));
                }
            }
        }
        Ok(rects)
    }

    /// Draw the background pixels and the focal pixes for left and right eye.
    pub fn draw(self: &mut Self, ui: &mut egui::Ui) {
        if self.arrays.background.nrows() != self.grid_size {
            self.gen_pixel_arrays()
        };

        Frame::dark_canvas(ui.style())
            .outer_margin(Margin::from(0.0)) // TODO: look into eliminating visible margin
            // (negative number works but what are the downsides?)
            .show(ui, |ui| {
                // Determine size of drawing surface: full screen
                let desired_size = ui.available_size_before_wrap();
                let (_id, rect) = ui.allocate_space(desired_size);
                // Determine starting coords to end up with a centered drawing
                let to_screen = emath::RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                    rect,
                );

                // how many pixels do we need?
                let drawsize: f32 = self.grid_size as f32 * self.pixel_size as f32;
                let rel_size_x = drawsize / desired_size[0] / 2.; // how wide is half a drawing?
                let rel_size_y = drawsize / desired_size[1] / 2.; // how high is half a drawing?
                let screen_offset = pos2(0.5 - rel_size_x, 0.0 - rel_size_y);
                let origin = to_screen * screen_offset;

                if self.debug.draw_left == true {
                    let left = self.draw_background(Eye::Left, &origin).unwrap_or_default(); // TODO error handling
                    ui.painter().extend(left);
                }
                if self.debug.draw_right == true {
                    let right = self
                        .draw_background(Eye::Right, &origin)
                        .unwrap_or_default(); // TODO error handling
                    ui.painter().extend(right);
                }

                // ui.painter().extend(focal_left);
                // ui.painter().extend(focal_right);
            });
    }
}
