use crate::shared::anaglyph::AnaglyphColor;
use egui::{emath, epaint::PathShape, pos2, vec2, Frame, Margin, Rect, Stroke};

/// Shows a menu to calibrate the colors used in the anaglyph painting.
/// Different glasses for viewing anaglyphs exist, user must be able to
/// set colors for optimal effect.
/// TODO: there is currently no option to permanently save calibration data.
pub fn calibrate(color: &mut AnaglyphColor, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
            ui.label("Calibrate the colors for your anaglyph glasses so each color is clearly visible to one eye, but hardly visible to the other. When properly calibrated the two diamonds may appear as one when seen through the glasses.");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Left eye");
                ui.color_edit_button_srgba(&mut color.left);
                ui.add_space(ui.available_width() / 3.);

                ui.color_edit_button_srgba(&mut color.right);
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
                        PathShape::convex_polygon(points, color.left, Stroke::NONE)
                    };
                    let right_diamond = {
                        let points = gen_points(-0.2);
                        PathShape::convex_polygon(points, color.right, Stroke::NONE)
                    };

                    ui.painter().add(left_diamond);
                    ui.painter().add(right_diamond);

                });

            ui.horizontal(|ui| {
                if ui.button("Swap").clicked() {
                    let tmp = color.left;
                    color.left = color.right;
                    color.right = tmp;
                }
            });


        });
}
