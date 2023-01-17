use eframe::{emath, epaint::RectShape};
use egui::{pos2, style::Margin, vec2, Color32, Frame, Rect, Shape};
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

enum Position {}

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
        self.draw_focal(Direction::Left);
    }

    /// Draws the background from a randomly generated self.bg_pixel_array.
    /// Draw_focal needs to be called beforehand so the pixels in the focal
    /// point are removed from the background array.
    fn draw_background(self: &mut Self) -> Result<Vec<Shape>, std::io::Error> {
        let mut shapes = vec![];
        Ok(shapes)
    }

    /// Draws a diamond shaped focal point. Removes the datapoints for the focal point from
    /// the pixel_array used for the background. This creates the illusion of the focal point
    /// obscuring its background.
    ///
    /// Focal offset creates the illusion of the focal point being in front of the background.
    /// To achieve this effect, the focal point is shifted slightly to the center of vision
    /// relative to the background. The brain interprets this as the object being closer.
    fn draw_focal(self: &mut Self, eye: Direction) -> () {
        let (color, bg_offset, focal_offset) = match eye {
            Direction::Left => (
                self.color.left,
                (self.bg_offset as f32 * -0.5) as i32, // bg moves further apart
                (self.focal_offset as f32 * 0.5) as i32, // focal is moved closer
            ),
            Direction::Right => (
                self.color.right,
                (self.bg_offset as f32 * 0.5) as i32,
                (self.focal_offset as f32 * -0.5) as i32,
            ),
            _ => (self.color.left, 0, 0),
        };

        // Return a tuple giving the position of the focal point relative to the background
        let focal_loc = match self.focal_position {
            Direction::Up => (0.5, 0.25),
            Direction::Down => (0.5, 0.75),
            Direction::Left => (0.25, 0.5),
            Direction::Right => (0.75, 0.5),
        };

        // Determine the starting point for 'drawing' the diamond shape in the array
        let focal_size = self.grid_size as f32 * self.focal_size_rel; // 35
                                                                      // let focal_pixel_count = focal_size / self.pixel_size as f32; // 35 / 1
        let x_min = self.grid_size as f32 * focal_loc.0;
        let y_min = self.grid_size as f32 * focal_loc.1 - focal_size / 2.;
        let focal_size = focal_size as usize;

        // create a matrix containing the diamond shape focal point
        let mut diamond = vec![];
        // step means: how many extra pixels to draw on each subsequent row of the matrix
        // by incrementing the amount of pixels to draw, a triangle shape is created
        let step = (focal_size / (focal_size / 2)) as usize;
        // create a list of the number of pixels to draw on each row
        for i in (0..focal_size).step_by(step) {
            diamond.push(i)
        }
        // finally, extend the list with itself reversed,
        // effectively appending an upside down triangle
        let mut rev = diamond.clone();
        rev.reverse();
        diamond.extend(rev);

        let zip = zip(0..diamond.len(), diamond.iter());
        for (y, pixels) in zip {
            let y = y_min as usize + y; // == y_min + current row
            let x = x_min as usize - pixels / 2; // half of the pixels to be drawn
                                                 // must be drawn left of center
            for x in x..(x + pixels) {
                self.pixel_array[[x + focal_offset as usize, y]] = 0; // remove the diamond from the bg array
            }
        }
    }

    /// Draw the background pixels and the focal pixes for left and right eye.
    // How?
    // Per anaglyph:
    //  Generate pixel array for the background
    //  Generate pixel array for the shape of the focal point
    //  Remove focal point shape from background (with slight offset)
    //  Generate focal array in the shpae of the focal point
    // Per frame:
    //  Draw the bg array (func takes ui, left/right eye)
    //  Draw the focal array (func takes ui, left/right eye)
    pub fn draw(self: &mut Self, ui: &mut egui::Ui) {
        let color = Color32::from_additive_luminance(196);
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

                // We need two glyphs in two distinct colors (left and right eye)
                // After we create the glyphs, we punch holes in them.
                // Then we draw the focal points.

                // Create set of rectangles and push them to the screen
                let mut rects = vec![];
                // create rows
                for y in 0..self.grid_size {
                    // fill in each row
                    for x in 0..self.grid_size {
                        // only create a 'pixel' if the random seed is 1 for this coord
                        if self.pixel_array[[x, y]] == 1 {
                            let coords_min =
                                vec2((x * self.pixel_size) as f32, (y * self.pixel_size) as f32);
                            let coords_max =
                                coords_min + vec2(self.pixel_size as f32, self.pixel_size as f32);
                            let sq = RectShape::filled(
                                Rect {
                                    min: origin + coords_min,
                                    max: origin + coords_max,
                                },
                                0.0,
                                color,
                            );
                            rects.push(Shape::Rect(sq));
                        }
                    }
                }

                ui.painter().extend(rects);
            });
    }
}
