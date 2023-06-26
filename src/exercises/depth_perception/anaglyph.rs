use eframe::emath;
use egui::{
    emath::RectTransform, epaint::CircleShape, pos2, style::Margin, vec2, Color32, Frame, Pos2,
    Rect, Shape, Stroke,
};
use rand::Rng;

use crate::{
    exercises::Direction, shared::anaglyph::AnaglyphColor,
    shared::asset_loader::exercise_config::depth_perception::DepthPerceptionExercise, widgets,
};

/// Struct for anaglyph images, in this case a number of rings and an indicator arrow.
/// `draw()` draws both the rings and the indicator arrow, according to a configuration.
/// The offset creates the illusion of depth. One if the rings has a different offset,
/// creating the impression that it is either in front or in back of the other rings.
/// The challenge for the user is to move the arrow to indicate the ring that stands out.
pub struct Anaglyph {
    pub config: DepthPerceptionExercise,
    pub circles: usize,
    pub target_index: usize, // The circle that has a different depth
    pub arrow_position: usize,
    offset: f32,
    target_offset: f32,
    circle_radius: f32,
    pub color: AnaglyphColor,
}

impl Default for Anaglyph {
    fn default() -> Self {
        Self {
            config: DepthPerceptionExercise::default(),
            circles: 5,
            target_index: 2,
            arrow_position: 0,
            offset: 0.005,
            target_offset: 0.02,
            circle_radius: 0.05,
            color: AnaglyphColor::default(),
        }
    }
}

impl Anaglyph {
    /// Convert the config values to usable parameters
    pub fn next(&mut self) {
        // We need an RNG for much of the following.
        let mut rng = rand::thread_rng();

        // Randomize which circle is the target.
        self.target_index = rng.gen_range(0..self.circles);

        // Convert circle size to a percentage of max screen height/width
        self.circle_radius = self.config.circle_size as f32 * 0.01;

        // Set random offset
        // The offset parameter is a % of a 10% fraction of the circle radius.
        // This makes sure the offset stays within the same relative bounds for
        // different screen sizes and resolutions.
        //
        // Note: sometimes the offset is flipped into a negative in order to
        // switch between convergent and divergent eye movement
        let offset_min = self.config.offset_min as f32 * self.circle_radius * 0.1;
        let offset_max = self.config.offset_max as f32 * self.circle_radius * 0.1;
        self.offset = match rng.gen() {
            true => rng.gen_range(offset_min..offset_max),
            false => rng.gen_range(offset_min..offset_max) * -1.0,
        };
        // Set target offset
        // This offset parameter is % of a 3% fraction of the circle radius.
        // The 3% number was arrived at empirically, it has no special significance.
        let min_diff = self.config.offset_target_variance_min as f32 * self.circle_radius * 0.03;
        let max_diff = self.config.offset_target_variance_max as f32 * self.circle_radius * 0.03;
        let offset_diff = rng.gen_range(min_diff..max_diff);
        self.target_offset = match rng.gen() {
            true => self.offset + offset_diff,
            false => self.offset - offset_diff,
        };
    }

    /// Draws an anaglyph circle: two circles in different colors separated by a given x offset.
    fn draw_circle(&mut self, pos: Pos2, size: f32, offset: f32) -> Vec<Shape> {
        let left = CircleShape::stroke(
            pos - vec2(offset, 0.),
            size,
            Stroke::new(size * 0.1, self.color.left), // Stroke is 10% of radius
        );
        let right = CircleShape::stroke(
            pos + vec2(offset, 0.),
            size,
            Stroke::new(size * 0.1, self.color.right), // Stroke is 10% of radius
        );

        // Return as vec of shapes
        vec![Shape::Circle(left), Shape::Circle(right)]
    }

    /// Draw a row of anagylph circles in the center of a given space.
    ///
    /// # Parameters
    /// rect: a Rectangle of some given size, constituting the space to draw in.
    /// to_screen: a RectTranform to calculate absolute pixel positions from relative
    fn draw_circle_row_and_arrow(&mut self, rect: Rect, to_screen: RectTransform) -> Vec<Shape> {
        // Where on the screen do we start drawing from left to right?
        // Note: the circle radius is half the width of a circle.
        // So the total width of a circle is radius * 2
        // And so the total width of the row is [`number of circles * radius * 2)`]
        let margin = self.circle_radius * 1.5;
        let x_min = 0.5 - (self.circles as f32 * self.circle_radius);

        // Calculate absolute pixel sizes
        let largest_side = rect.width().max(rect.height());
        let offset_absolute = self.offset * largest_side;
        let target_offset_absolute = self.target_offset * largest_side;
        let circle_size_absolute = self.circle_radius * largest_side;

        // Anonymous function to calculate position.
        let calc_pos = |i: usize, radius| {
            let x = x_min + i as f32 * (radius + margin);
            let y = 0.5;
            to_screen * pos2(x, y)
        };

        // Gather shapes into a vec
        let mut shapes = vec![];

        // Draw circles for all positions
        for i in 0..self.circles {
            let pos = calc_pos(i, self.circle_radius);
            // All shapes get the same offset, except for the 'target shape'.
            // The target shape gets the target offset, setting it apart from
            // the other shapes visually.
            let circle = match i {
                i if i == self.target_index => {
                    self.draw_circle(pos, circle_size_absolute, target_offset_absolute)
                }
                _ => self.draw_circle(pos, circle_size_absolute, offset_absolute),
            };
            shapes.extend(circle);
        }

        shapes
    }

    fn draw_arrow(&self, to_screen: RectTransform) -> Shape {
        let x_min = 0.5 - (self.circles as f32 * self.circle_radius);
        let arrow_pos = pos2(
            x_min + (self.arrow_position as f32 * self.circle_radius * 2.5),
            0.7,
        );
        // Return arrow shape
        widgets::arrow_shape(arrow_pos, 3., &Direction::Up, to_screen, Color32::KHAKI)
    }

    /// Draws the exercise circles and indicator arrow
    pub fn draw(self: &mut Self, ui: &mut egui::Ui) {
        Frame::dark_canvas(ui.style())
            .outer_margin(Margin::from(0.0))
            .show(ui, |ui| {
                // Determine size of drawing surface: full screen
                let desired_size = ui.available_size_before_wrap();
                let (_id, rect) = ui.allocate_space(desired_size);
                // Determine starting coords to end up with a centered drawing
                let to_screen = emath::RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0),
                    rect,
                );

                let circles = self.draw_circle_row_and_arrow(rect, to_screen);
                let arrow = self.draw_arrow(to_screen);
                ui.painter().extend(circles);
                ui.painter().add(arrow);
            });
    }
}
