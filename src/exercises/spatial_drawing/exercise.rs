use egui::{emath, Color32, Frame, Pos2, Rect, Response, Sense, Ui};

use crate::shared::asset_loader::AppData;

use super::{SessionStatus, Transformation};

// All the exercise functions go here.
impl super::SpatialDrawing {
    /// Show the area where the user can draw and compare against the exercise
    pub fn ui_drawing(&mut self, ui: &mut Ui) -> egui::Response {
        // Set up
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        // Take care of line drawing
        if self.user_drawing.is_empty() {
            self.user_drawing.push(vec![]);
        }
        let current_line = self.user_drawing.last_mut().unwrap();
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.user_drawing.push(vec![]);
            response.mark_changed();
        }

        // Push shapes to painter
        // If reviewing the exercise, paint the solution using the right transformation
        if self.state == SessionStatus::Reviewing {
            let exercise_shapes = match self.puzzle_transform {
                Transformation::HMirror => {
                    self.puzzle.shapes_hmirror(&to_screen, 10., Color32::KHAKI)
                }
                Transformation::VMirror => {
                    self.puzzle.shapes_vmirror(&to_screen, 10., Color32::KHAKI)
                }
                Transformation::LeftTilt => {
                    self.puzzle
                        .shapes_tilt_left(&to_screen, 10., Color32::KHAKI)
                }
                Transformation::RightTilt => {
                    self.puzzle
                        .shapes_tilt_right(&to_screen, 10., Color32::KHAKI)
                }
            };
            painter.extend(exercise_shapes);
        }
        painter.extend(
            self.puzzle_grid
                .shapes(self.puzzle.size(), &to_screen, 5., Color32::WHITE),
        );

        let drawn_shapes = self
            .user_drawing
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });
        painter.extend(drawn_shapes);

        response
    }

    /// Shows the original drawing
    pub fn ui_example(&mut self, ui: &mut egui::Ui) -> Response {
        // Setup
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );

        // Push shapes to painter
        painter.extend(self.puzzle.shapes(&to_screen, 10., Color32::KHAKI));
        painter.extend(
            self.puzzle_grid
                .shapes(self.puzzle.size(), &to_screen, 5., Color32::WHITE),
        );

        response
    }

    /// Show the session (fullscreen)
    pub fn ui_exercise(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts, _: &AppData) {
        // Show controls at the top
        self.ui_controls(ui);

        // Left column shows example, right column is where user draws and reviews.
        ui.columns(2, |cols| {
            Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| self.ui_example(ui));
            Frame::dark_canvas(cols[1].style()).show(&mut cols[1], |ui| self.ui_drawing(ui));
        });
    }
}
