use eframe::{emath, epaint::RectShape};
use egui::{pos2, style::Margin, vec2, Color32, Frame, Pos2, Rect, Shape, Vec2};
use ndarray::Array2;
use ndarray_rand::{rand_distr::Binomial, RandomExt};
use perhabs::{Direction, PhError};
use std::iter::zip;

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

/// Struct for anaglyph images. Each image consists of a background and a focal point.
/// draw() draws both focal point and background in order, according to the pub variables.
pub struct Anaglyph {
    pub bg_offset: usize,
    pub pixel_size: usize,
    pub grid_size: usize,
    pixel_array: Array2<u64>,
    focal_array: Array2<u64>,
    mask_array: Array2<u64>,
    focal_offset: usize, // The offset creates the illusion of depth
    pub focal_size_rel: f32,
    pub focal_position: Direction, // Where is the focal point?
    pub color: AnaglyphColor,
}

impl Default for Anaglyph {
    fn default() -> Self {
        Self {
            pixel_size: 3,
            grid_size: 100,
            pixel_array: Array2::zeros((10, 10)),
            focal_array: Array2::zeros((10, 10)),
            mask_array: Array2::zeros((10, 10)),
            bg_offset: 0,
            focal_offset: 2,
            focal_size_rel: 0.35,
            focal_position: Direction::Up,
            color: AnaglyphColor::default(),
        }
    }
}

impl Anaglyph {
    /// Generate a random array of 1's and 0's. Then remove the 'background'
    /// to the focal glyphs to create depth illusion.
    pub fn gen_pixel_arrays(&mut self) {
        let distr = Binomial::new(1, 0.5).unwrap();
        self.pixel_array = Array2::random((self.grid_size, self.grid_size), distr);
        self.focal_array = Array2::random((self.grid_size, self.grid_size), distr);
        self.mask_array = Array2::zeros((self.grid_size, self.grid_size));
        // self.draw_focal(Direction::Left);
    }

    /// Draws the background from a randomly generated self.bg_pixel_array.
    /// Draw_focal needs to be called beforehand so the pixels in the focal
    /// point are removed from the background array.
    fn draw_background(
        self: &mut Self,
        eye: Direction,
        origin: &Pos2,
    ) -> Result<Vec<Shape>, PhError> {
        // Left/right image gets appropriate coloring and the offset value is split between them
        let (color, bg_offset) = match eye {
            Direction::Left => (self.color.left, self.bg_offset as f32 * -0.5),
            Direction::Right => (self.color.right, self.bg_offset as f32 * 0.5),
            _ => (self.color.left, 0.),
        };

        // Create rectangles and push them to a vec
        let mut rects = vec![];
        // create rows
        for y in 0..self.grid_size {
            // fill in each row
            for x in 0..self.grid_size {
                // only create a 'pixel' if the random seed is 1 for this coord
                if self.pixel_array[[x, y]] == 1 {
                    let coords_min = vec2(
                        (x * self.pixel_size) as f32 + bg_offset as f32,
                        (y * self.pixel_size) as f32,
                    );
                    let coords_max =
                        coords_min + vec2(self.pixel_size as f32, self.pixel_size as f32);
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
            }
        }
        Ok(rects)
    }

    /// Draws a diamond shaped focal point. Removes the datapoints for the focal point from
    /// the pixel_array used for the background. This creates the illusion of the focal point
    /// obscuring its background.
    ///
    /// Focal offset creates the illusion of the focal point being in front of the background.
    /// To achieve this effect, the focal point is shifted slightly to the center of vision
    /// relative to the background. The brain interprets this as the object being closer.
    fn draw_focal(self: &mut Self, eye: Direction, origin: &Pos2) -> Result<Vec<Shape>, PhError> {
        let (color, bg_offset, focal_offset) = match eye {
            Direction::Left => (
                self.color.left,
                self.bg_offset as f32 * -0.5,   // bg moves further apart
                self.focal_offset as f32 * 0.5, // focal moves closer
            ),
            Direction::Right => (
                self.color.right,
                self.bg_offset as f32 * 0.5,
                self.focal_offset as f32 * -0.5,
            ),
            _ => (self.color.left, 0., 0.),
        };

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
        let (x_min, y_min, focal_size) = (x_min as usize, y_min as usize, focal_size as usize);

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

        // For the next part we need to do some type conversions
        let pixel_size = self.pixel_size as f32;

        // Now we remove the focal pixels from the background array
        // and create the pixels (shapes) for the focal point
        let mut rects = vec![];
        let zip = zip(0..diamond.len(), diamond.iter());

        // A row has a row number and a number of pixels to fill it
        for (row, pixels) in zip {
            // TODO little type conversion problem
            // array indexing needs usize
            // all other stuff needs f32
            // So: convert all to f32
            // And cast to usize only for indexing

            // Determine x and y coords for the topleft of the first pixel in this row
            let y = y_min + row; // add the current row to the topmost coord of the glyph
            let x = x_min - pixels / 2; // Find the horizontal starting coord considering
                                        // half the pixels must be drawn left of center

            // Iterate over the number of pixels in this row
            for col in x..(x + pixels) {
                let x = col as f32;
                self.pixel_array[[(x + focal_offset) as usize, y]] = 0; // remove the diamond from the bg array

                // Do we need to draw a 'pixel' here?
                if self.focal_array[[col, row]] == 1 {
                    // draw focal pixels
                    let coords_min = vec2(
                        x * pixel_size + bg_offset + focal_offset,
                        y as f32 * pixel_size,
                    );
                    let coords_max =
                        coords_min + vec2(self.pixel_size as f32, self.pixel_size as f32);
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
            }
        }
        Ok(rects)
    }

    /// Draw the background pixels and the focal pixes for left and right eye.
    // How?
    // Per anaglyph:
    //  - Generate pixel arrays for bg and focal_point
    //  - Remove focal point shape from background (with slight offset)
    //  - Create focal pixels
    //  - Create bg pixels
    // Per frame:
    //  Draw the bg array (func takes ui, left/right eye)
    //  Draw the focal array (func takes ui, left/right eye)
    pub fn draw(self: &mut Self, ui: &mut egui::Ui) {
        if self.pixel_array.nrows() != self.grid_size {
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

                let bg_left = self
                    .draw_background(Direction::Left, &origin)
                    .unwrap_or_default(); // TODO error handling
                let bg_right = self
                    .draw_background(Direction::Right, &origin)
                    .unwrap_or_default(); // TODO error handling

                let focal_left = self.draw_focal(Direction::Left, &origin).unwrap();
                let focal_right = self.draw_focal(Direction::Right, &origin).unwrap();

                ui.painter().extend(bg_left);
                ui.painter().extend(bg_right);
                ui.painter().extend(focal_left);
                ui.painter().extend(focal_right);
            });
    }
}
