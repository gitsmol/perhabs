use egui::{
    emath,
    epaint::{CircleShape, CubicBezierShape, QuadraticBezierShape, RectShape},
    pos2, vec2, Align2, Color32, FontId, Mesh, Pos2, Rect, Response, Rounding, Sense, Shape,
    Stroke,
};

use crate::exercises::ExerciseStage;

use super::ContainerSearch;

const SECRET_COLOR: Color32 = Color32::LIGHT_YELLOW;
const UNOPENED_COLOR: Color32 = Color32::DARK_GREEN;
const OPENED_COLOR: Color32 = Color32::LIGHT_GRAY;
const RESULT_BG_COLOR: Color32 = Color32::from_black_alpha(200);
const GRID_MARGIN: f32 = 0.9;
const ROUNDING_FACTOR: f32 = 0.15;

impl ContainerSearch {
    pub(super) fn draw_session(&mut self, ui: &mut egui::Ui, debug: bool) -> Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
            response.rect,
        );
        let from_screen = to_screen.inverse();
        let abs_size = self.calculate_abs_size(&to_screen);

        self.draw_containers(&painter, &to_screen, abs_size);

        if self.stage == ExerciseStage::Response {
            self.handle_click(&response, &from_screen, abs_size);
        }

        if self.stage == ExerciseStage::Result {
            self.draw_result(&painter, ui, &to_screen);
        }

        if debug {
            self.draw_debug(&painter, &to_screen);
        }

        response
    }

    fn calculate_abs_size(&self, to_screen: &emath::RectTransform) -> f32 {
        (1.0 / self.grid_size as f32) * GRID_MARGIN * to_screen.scale().min_elem()
    }

    fn draw_containers(
        &self,
        painter: &egui::Painter,
        to_screen: &emath::RectTransform,
        abs_size: f32,
    ) {
        for pos in &self.containers.unopened {
            painter.add(self.create_rect_shape(pos, UNOPENED_COLOR, to_screen, abs_size));
        }

        for pos in &self.containers.opened {
            painter.add(self.create_opened_box_shape(pos, OPENED_COLOR, to_screen, abs_size));
        }
    }

    fn draw_debug(&mut self, painter: &egui::Painter, to_screen: &emath::RectTransform) {
        let abs_size = self.calculate_abs_size(&to_screen);

        // draw secrets
        let shape = CircleShape::filled(
            to_screen * self.containers.secret,
            abs_size / 3.,
            Color32::from_additive_luminance(100),
        );
        painter.add(shape);

        for secret in &self.containers.found_secrets {
            let shape =
                CircleShape::filled(to_screen * *secret, abs_size / 3., Color32::from_gray(200));
            painter.add(shape);
        }
    }

    fn create_rect_shape(
        &self,
        pos: &Pos2,
        color: Color32,
        to_screen: &emath::RectTransform,
        abs_size: f32,
    ) -> egui::Shape {
        let rect = Rect::from_center_size(to_screen * *pos, vec2(abs_size, abs_size));
        egui::Shape::Rect(RectShape::filled(
            rect,
            Rounding::same(abs_size * ROUNDING_FACTOR),
            color,
        ))
    }

    fn create_opened_box_shape(
        &self,
        pos: &Pos2,
        color: Color32,
        to_screen: &emath::RectTransform,
        abs_size: f32,
    ) -> Vec<Shape> {
        let rect = Rect::from_center_size(to_screen * *pos, vec2(abs_size, abs_size));
        let outer_shape = egui::Shape::Rect(RectShape::filled(
            rect,
            Rounding::same(abs_size * ROUNDING_FACTOR),
            color,
        ));
        let mut shapes: Vec<Shape> = vec![outer_shape];

        let depth = abs_size * 0.2;
        let stroke = Stroke::new(1.0, Color32::from_black_alpha(50));

        let left_top_inside = rect.left_top() + vec2(depth, depth);
        let left_bottom_inside = rect.left_bottom() + vec2(depth, -depth);
        let right_top_inside = rect.right_top() - vec2(depth, -depth);
        let right_bottom_inside = rect.right_bottom() - vec2(depth, depth);

        let lines = [
            Shape::line_segment([rect.left_top(), left_top_inside], stroke),
            Shape::line_segment([rect.left_bottom(), left_bottom_inside], stroke),
            Shape::line_segment([rect.right_top(), right_top_inside], stroke),
            Shape::line_segment([rect.right_bottom(), right_bottom_inside], stroke),
        ];

        let inside_rect = RectShape::filled(
            Rect::from_two_pos(left_top_inside, right_bottom_inside),
            Rounding::ZERO,
            Color32::from_black_alpha(50),
        );
        shapes.push(Shape::Rect(inside_rect));

        shapes.extend(lines);
        shapes
    }

    fn draw_result(
        &self,
        painter: &egui::Painter,
        ui: &mut egui::Ui,
        to_screen: &emath::RectTransform,
    ) {
        let bg_rect = Rect::from_two_pos(to_screen * pos2(0.4, 0.4), to_screen * pos2(0.6, 0.6));
        painter.add(RectShape::filled(
            bg_rect,
            Rounding::same(3.0),
            RESULT_BG_COLOR,
        ));

        // draw secrets
        let shape = CircleShape::filled(
            to_screen * self.containers.secret,
            self.calculate_abs_size(to_screen) / 4.,
            SECRET_COLOR,
        );
        painter.add(shape);

        if let Some(result) = self.round_score.last() {
            let (symbol, color) = match result {
                &true => ("\u{2714}", Color32::GREEN),
                &false => ("\u{2716}", Color32::RED),
            };

            ui.painter().text(
                bg_rect.center(),
                Align2::CENTER_BOTTOM,
                symbol,
                FontId::proportional(50.),
                color,
            );
        }
    }

    fn handle_click(
        &mut self,
        response: &Response,
        from_screen: &emath::RectTransform,
        abs_size: f32,
    ) {
        if response.clicked() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen * pointer_pos;
                let clickable_area_size = abs_size / 2.0 * from_screen.scale();
                if let Some(pos) =
                    self.grid
                        .match_coords(self.grid_size, canvas_pos, clickable_area_size)
                {
                    self.evaluate_response(&pos.to_owned());
                }
            }
        }
    }
}
