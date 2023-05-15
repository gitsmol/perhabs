use egui::{emath, Color32, Pos2, Rect, Response, Sense};

use super::painters::Puzzle;

// All the selector functions go here.
impl super::SpatialDrawing {
    /// Shows an editable exercise
    pub fn ui_mini(&self, ui: &mut egui::Ui, exercise: &Puzzle) -> Response {
        // Setup
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());

        if response.clicked() {
            response.mark_changed();
        }

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );

        painter.extend(exercise.shapes(&to_screen, 10., Color32::KHAKI));
        painter.extend(self.puzzle_grid.shapes(&to_screen, 5., Color32::WHITE));

        response
    }
}
