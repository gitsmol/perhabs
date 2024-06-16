use egui::{
    emath, epaint::RectShape, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Response, Rounding,
    Sense,
};

use crate::{exercises::ExerciseStage, shared::AppData};

use super::NumberedSquares;

impl NumberedSquares {
    pub(super) fn draw_debug(&mut self, ui: &mut egui::Ui) -> Response {
        // Setup
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        // Push shapes to painter
        let abs_size: f32 = {
            let margin = 0.9;
            (1.0 / self.grid_size as f32) * margin * to_screen.scale().min_elem()
            // 0.09 * to_screen.scale().min_elem()
        };
        for col in self.grid.get_all_coords(self.grid_size) {
            for pos in col.iter() {
                let rect = Rect::from_center_size(to_screen * *pos, vec2(abs_size, abs_size));
                let color = {
                    if self.answers.response.contains(pos) {
                        Color32::YELLOW
                    } else {
                        match self.answers.sequence.contains(pos) {
                            true => Color32::DARK_GREEN,
                            false => Color32::LIGHT_GRAY,
                        }
                    }
                };
                painter.add(egui::Shape::Rect(RectShape::filled(
                    rect,
                    Rounding::same(abs_size * 0.15),
                    color,
                )));
                painter.debug_text(
                    to_screen * *pos,
                    Align2::CENTER_CENTER,
                    Color32::WHITE,
                    format!("{:?}", pos),
                );
            }
        }

        // Show pointer position
        let pointer_pos = match response.hover_pos() {
            Some(pos) => pos,
            None => Pos2::ZERO,
        };

        let canvas_pos = from_screen * pointer_pos;
        painter.debug_text(
            pointer_pos,
            Align2::CENTER_CENTER,
            Color32::WHITE,
            format!("{:?} - {:?}", canvas_pos, pointer_pos),
        );

        // Show quad position
        // highlighting
        let clickable_area_size = 1.0 / self.grid_size as f32;
        let rect = Rect::from_center_size(pointer_pos, vec2(abs_size, abs_size));
        painter.debug_rect(rect, Color32::RED, format!("{:?}", canvas_pos));

        for pos in self.answers.sequence.iter() {
            let pos_on_screen = to_screen * pos.to_owned();
            ui.painter().text(
                pos_on_screen,
                Align2::CENTER_CENTER,
                format!("{:?}", pos),
                FontId::monospace(12.0),
                Color32::from_additive_luminance(240),
            );
        }

        if response.clicked() {
            // Show pointer position
            let pointer_pos = match response.interact_pointer_pos() {
                Some(pos) => pos,
                None => Pos2::ZERO,
            };

            let canvas_pos = from_screen * pointer_pos;
            let clickable_area_size = abs_size / 2.0 * from_screen.scale();
            if let Some(pos) =
                self.grid
                    .match_coords(self.grid_size, canvas_pos, clickable_area_size)
            {
                self.answers.response.push(pos.to_owned());
            }
        }

        response
    }

    /// Show squares on the grid, show challenge, show result
    pub(super) fn draw_session(&mut self, ui: &mut egui::Ui) -> Response {
        // Setup
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        // Push shapes to painter
        let abs_size: f32 = {
            let margin = 0.9;
            (1.0 / self.grid_size as f32) * margin * to_screen.scale().min_elem()
        };
        for pos in &self.answers.sequence {
            let rect = Rect::from_center_size(to_screen * *pos, vec2(abs_size, abs_size));
            painter.add(egui::Shape::Rect(RectShape::filled(
                rect,
                Rounding::same(abs_size * 0.15),
                Color32::DARK_GREEN,
            )));
        }

        // Draw challenge: show number sequence overlayed on grid squares
        // Draw numbers on top of the given positions. Enumerates the items in the positions
        // vector and then draws a number for each position.
        if self.stage == ExerciseStage::Challenge {
            debug!("Status == Challenge");
            for (i, pos) in self.answers.sequence.iter().enumerate() {
                let pos_on_screen = to_screen * pos.to_owned();
                ui.painter().text(
                    pos_on_screen,
                    Align2::CENTER_CENTER,
                    format!("{}", i + 1),
                    FontId::monospace(25.0),
                    Color32::from_additive_luminance(240),
                );
                debug!("Drawing number {i} at {:?}", pos);
            }
        }

        // Show the result of the challenge/response-stages
        if self.stage == ExerciseStage::Result {
            let bg_rect =
                Rect::from_two_pos(to_screen * pos2(0.4, 0.4), to_screen * pos2(0.6, 0.6));
            painter.add(RectShape::filled(
                bg_rect,
                Rounding::same(3.0),
                Color32::from_black_alpha(200),
            ));
            if self.evaluate_response() {
                ui.painter().text(
                    bg_rect.center(),
                    Align2::CENTER_BOTTOM,
                    "\u{2714}",
                    FontId::proportional(50.),
                    Color32::GREEN,
                );
            } else {
                ui.painter().text(
                    bg_rect.center(),
                    Align2::CENTER_BOTTOM,
                    "\u{2716}",
                    FontId::proportional(50.),
                    Color32::RED,
                );
            }
        }

        // Early return if we don't allow response.
        if self.stage != ExerciseStage::Response {
            return response;
        }

        // Find the clicked square.
        // Wrap this in `clicked()` so only one click is registered.
        if response.clicked() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                let clickable_area_size = abs_size / 2.0 * from_screen.scale();
                if let Some(pos) =
                    self.grid
                        .match_coords(self.grid_size, canvas_pos, clickable_area_size)
                {
                    self.answers.response.push(pos.to_owned());
                }
            }
        }

        response
    }
}
