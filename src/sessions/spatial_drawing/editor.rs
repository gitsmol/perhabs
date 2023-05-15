use egui::{emath, Color32, Pos2, Rect, Response, Sense};

// All the editor functions go here.
impl super::SpatialDrawing {
    /// Shows an editable exercise
    pub fn ui_editable(&mut self, ui: &mut egui::Ui) -> Response {
        // Setup
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        // On click, get pointer position. If user clicks a guide circle,
        // feed its position to the line editor.
        if response.clicked() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                if let Some(pos) = self.puzzle_grid.match_coords(canvas_pos) {
                    debug!("Adding to line edit: {:?}", pos);
                    self.puzzle.edit(*pos);
                }
            }
        }

        painter.extend(self.puzzle.shapes(&to_screen, 10., Color32::KHAKI));
        painter.extend(self.puzzle_grid.shapes(&to_screen, 5., Color32::WHITE));

        response
    }

    /// Save a drawing.
    pub fn save(&self) {}

    /// Select a drawing.
    pub fn select(&self) {}
}
