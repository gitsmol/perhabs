use crate::{modules::asset_loader::AppData, wm::sessionman::Exercise};
use egui::{
    emath::{self, RectTransform},
    epaint::RectShape,
    pos2, Color32, Frame, Pos2, Rect, Rounding, Sense, Shape, Stroke,
};
use perhabs::Direction;
use rand::{seq::SliceRandom, Rng};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VisSaccades {
    session: bool,
    arrow_pos: Option<Pos2>,
    arrow_direction: Direction,
}

impl Default for VisSaccades {
    fn default() -> Self {
        Self {
            session: false,
            arrow_pos: None,
            arrow_direction: Direction::Up,
        }
    }
}

impl VisSaccades {
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        if ui.button("New").clicked() {
            self.new_arrow_pos();
        }
        ui.label(format!("Pos: {:?}", self.arrow_pos));
    }

    fn arrow_painter(&self, ui: &mut egui::Ui) {
        // Set up
        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            Sense::focusable_noninteractive(),
        );
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );

        if let Some(pos) = self.arrow_pos {
            let shapes = self.arrow_shape(pos, to_screen);
            painter.extend(shapes);
        }
    }

    fn new_arrow_pos(&mut self) {
        if let Some(direction) = vec![
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ]
        .choose(&mut rand::thread_rng())
        {
            self.arrow_direction = *direction;
        }

        let mut rng = rand::thread_rng();
        let x: f32 = rng.gen_range(0.01..0.95);
        let y: f32 = rng.gen_range(0.01..0.95);
        self.arrow_pos = Some(pos2(x, y));
    }

    fn calc_arrow_pos(&mut self) {
        self.arrow_pos = Some(pos2(0.5, 0.5));
    }

    /// Return a vec containing shapes suitable for egui::Painter.
    /// The shapes make up an arrow.
    fn arrow_shape(&self, pos: Pos2, to_screen: RectTransform) -> Vec<Shape> {
        let measure = 0.02;
        let half = measure / 2.;
        // The arrowhead first
        let arrowhead_points = match self.arrow_direction {
            Direction::Up => vec![
                to_screen * pos2(pos.x, pos.y),                  // The tip
                to_screen * pos2(pos.x + half, pos.y + measure), // Right
                to_screen * pos2(pos.x - half, pos.y + measure), // Left
            ],
            Direction::Down => vec![
                to_screen * pos2(pos.x, pos.y),                  // The tip
                to_screen * pos2(pos.x + half, pos.y - measure), // Right
                to_screen * pos2(pos.x - half, pos.y - measure), // Left
            ],
            Direction::Left => vec![
                to_screen * pos2(pos.x, pos.y),
                to_screen * pos2(pos.x - measure, pos.y + half),
                to_screen * pos2(pos.x, pos.y + measure),
            ],
            Direction::Right => vec![
                to_screen * pos2(pos.x, pos.y),
                to_screen * pos2(pos.x + measure, pos.y + half),
                to_screen * pos2(pos.x, pos.y + measure),
            ],
        };
        let arrowhead = Shape::convex_polygon(arrowhead_points, Color32::KHAKI, Stroke::NONE);

        // Now the tail of the arrow
        let arrow_tail_points = match self.arrow_direction {
            Direction::Up => Rect::from_two_pos(
                to_screen * pos2(pos.x - 0.005, pos.y + 0.01),
                to_screen * pos2(pos.x + 0.005, pos.y + 0.035),
            ),
            Direction::Down => Rect::from_two_pos(
                to_screen * pos2(pos.x - 0.01, pos.y + 0.02),
                to_screen * pos2(pos.x + 0.01, pos.y + 0.04),
            ),
            Direction::Left => Rect::from_two_pos(
                to_screen * pos2(pos.x - 0.01, pos.y + 0.02),
                to_screen * pos2(pos.x + 0.01, pos.y + 0.04),
            ),
            Direction::Right => Rect::from_two_pos(
                to_screen * pos2(pos.x - 0.01, pos.y + 0.02),
                to_screen * pos2(pos.x + 0.01, pos.y + 0.04),
            ),
        };

        let arrow_tail = Shape::Rect(RectShape::filled(
            arrow_tail_points,
            Rounding::none(),
            Color32::KHAKI,
        ));

        vec![arrowhead, arrow_tail]
    }
}

impl Exercise for VisSaccades {
    fn name(&self) -> &'static str {
        "Tracking (Saccades)"
    }

    fn description(&self) -> &'static str {
        "Quickly scan the screen and respond."
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut tts::Tts) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_controls(ui);
            Frame::dark_canvas(ui.style()).show(ui, |ui| self.arrow_painter(ui));
        });
    }
}
