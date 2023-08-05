use egui::{emath, epaint::RectShape, pos2, Color32, Pos2, Rect, Rounding, Sense, Shape, Stroke};

use crate::{exercises::ExerciseStatus, shared::pos3::Pos3};

impl super::SpatialHearing {
    /// How do we represent a 3D space in a simplistic 2D fashion?
    /// In this case, we use a so called 'single point perspective' style of drawing.
    /// This means that there is a single vanishing point at the center of the drawing.
    /// All lines along all axes converge at this point. Consequently, to represent
    /// something as 'further away' (ie higher on the Z-axis) we need to adjust the
    /// X- and Y-axis too.
    ///
    /// For example, if (0, 0) is the vanishing point of a 2 dimensional system, all
    /// operations that bring Z closer to 1 must bring X and Y closer to 0. To the
    /// point where they are (asymptotically close to) 0 when Z is 1.
    pub fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // Set up
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        // Create a transform mapping the available space on a rectangle,
        // In this case, we make 0.0 the center of the screen. This works well with
        // the single point perspective described below.
        let to_screen = emath::RectTransform::from_to(
            Rect::from_x_y_ranges(-1.0..=1.0, -1.0..=1.0),
            response.rect,
        );

        // This anonymous function performs the single point perspective conversion inline
        let perspective_transform = |point: Pos3| -> Pos2 {
            let depth_factor = 1.0 - point.z();
            let x = point.x() * depth_factor;
            let y = point.y() * depth_factor;

            pos2(x, y)
        };

        //
        // Painting the scene
        //

        // Paint a frame at Z = 0. This is the frame closest to the viewer.
        let frame_z0_top_left = Pos3::new(-0.9, -0.9, 0.0);
        let frame_z0_bottom_right = Pos3::new(0.9, 0.9, 0.0);
        let frame_z0 = Rect::from_two_pos(
            to_screen * perspective_transform(frame_z0_top_left),
            to_screen * perspective_transform(frame_z0_bottom_right),
        );
        painter.add(RectShape::stroke(
            frame_z0,
            Rounding::none(),
            Stroke::new(1.0, Color32::YELLOW),
        ));

        // Paint a frame at Z = 1. This is the frame furthest from the viewer.
        let frame_z1_top_left = Pos3::new(-0.9, -0.9, 0.9);
        let frame_z1_bottom_right = Pos3::new(0.9, 0.9, 0.9);
        let frame_z1 = Rect::from_two_pos(
            to_screen * perspective_transform(frame_z1_top_left),
            to_screen * perspective_transform(frame_z1_bottom_right),
        );
        painter.add(RectShape::stroke(
            frame_z1,
            Rounding::none(),
            Stroke::new(1.0, Color32::YELLOW),
        ));

        // Paint lines between the corners of these frames to suggest a 3D space
        let lines = vec![
            Shape::line_segment(
                [frame_z0.left_top(), frame_z1.left_top()],
                Stroke::new(1.0, Color32::YELLOW),
            ),
            Shape::line_segment(
                [frame_z0.right_top(), frame_z1.right_top()],
                Stroke::new(1.0, Color32::YELLOW),
            ),
            Shape::line_segment(
                [frame_z0.left_bottom(), frame_z1.left_bottom()],
                Stroke::new(1.0, Color32::YELLOW),
            ),
            Shape::line_segment(
                [frame_z0.right_bottom(), frame_z1.right_bottom()],
                Stroke::new(1.0, Color32::YELLOW),
            ),
        ];
        painter.extend(lines);

        //
        // Painting the sound sources
        //
        let sq_size_offset = pos2(0.05, 0.05);

        for source in &mut self.sound_sources {
            // Calculate the sound source rects. `to_screen` denormalizes the
            // positions to on screen pixels. `perspective transform` shrinks
            // the x and y dimensions of each object according to its position
            // on the z-axis.
            let rect = Rect::from_two_pos(
                to_screen * perspective_transform(source.pos3 - sq_size_offset),
                to_screen * perspective_transform(source.pos3 + sq_size_offset),
            );
            source.rect = Some(rect);
        }

        // Now paint the sound source rects
        for source in &self.sound_sources {
            if let Some(rect) = source.rect {
                painter.add(RectShape::stroke(
                    rect,
                    Rounding::none(),
                    Stroke::new(1.0, Color32::BLUE),
                ));
            }
        }

        // If we are in result mode, paint the given answer and the right answer
        if self.status == ExerciseStatus::Result {
            // Unwrap answer and response or return (sort of a guard clause)
            let Some(answer) = &self.answer else {return response};
            let Some(response) = &self.response else {return response};

            // Paint the answer in filled blue
            if let Some(rect) = answer.rect {
                painter.add(RectShape::filled(rect, Rounding::none(), Color32::BLUE));
            };

            // Paint the response in filled red or green
            if let Some(rect) = response.rect {
                let color = {
                    if answer.coords == response.coords {
                        Color32::GREEN
                    } else {
                        Color32::RED
                    }
                };
                painter.add(RectShape::filled(rect, Rounding::none(), color));
            };
        }

        response
    }
}
