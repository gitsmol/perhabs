use crate::windowman::{AppWin, View};
use chrono::{Local, Timelike};
use eframe::epaint::{self, CircleShape};
use egui::{emath, pos2, vec2, Color32, Frame, Pos2, Rect, Stroke};

use std::f32::consts::TAU;

pub struct Clock {}

impl Default for Clock {
    fn default() -> Self {
        Self {}
    }
}

impl Clock {}

impl AppWin for Clock {
    fn name(&self) -> &'static str {
        "Clock"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, mut spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, spk));
    }
}

impl View for Clock {
    fn ui(&mut self, ui: &mut egui::Ui, spk: &mut tts::Tts) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let desired_size = ui.available_width() * vec2(1., 1.);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

            let circ_center = to_screen * pos2(0.5, 0.);
            let radius = 100.;
            let circleshape = CircleShape::stroke(circ_center, radius, Stroke::new(2., color));
            let circle = epaint::Shape::Circle(circleshape);
            ui.painter().add(circle);

            let marks = draw_marks(circ_center, radius, color);
            ui.painter().extend(marks);

            let hands = draw_hands(circ_center, radius, color);
            ui.painter().extend(hands);
        });
    }
}

fn draw_marks(center: Pos2, radius: f32, color: Color32) -> Vec<epaint::Shape> {
    let angle = |period, time: f32| TAU * (time.rem_euclid(period) / period) as f32 - TAU / 4.0;
    let coord = |angle: f32, radius: f32| {
        pos2(
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
        )
    };
    let mut shapes = vec![];
    for h in 0..12 {
        let _angle = angle(12., h as f32);
        let _inner_coord = coord(_angle, radius * 0.9);
        let _outer_coord = coord(_angle, radius);
        let shape =
            epaint::Shape::line_segment([_inner_coord, _outer_coord], Stroke::new(2.0, color));
        shapes.push(shape)
    }
    shapes
}

/// Draw hands of the clock
fn draw_hands(center: Pos2, radius: f32, color: Color32) -> Vec<epaint::Shape> {
    let dt = Local::now(); // We need this to get nanoseconds (to get a smooth movement on the sec hand)
    let sfm = dt.num_seconds_from_midnight() as f32; // we need this to get fractional hours

    // Closures to calculate the angle and polar coordinates for the hands
    let angle = |period, time: f32| TAU * (time.rem_euclid(period) / period) as f32 - TAU / 4.0;
    let coord = |angle: f32, radius: f32| {
        pos2(
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
        )
    };

    // The hour hand
    let hour_angle = angle(12. * 60. * 60., sfm);
    let hour_coord = coord(hour_angle, radius * 0.6);
    let hour_hand = epaint::Shape::line_segment([center, hour_coord], Stroke::new(3.0, color));

    // The minute hand
    let minute_angle = angle(60. * 60., sfm);
    let minute_coord = coord(minute_angle, radius * 0.85);
    let minute_hand = epaint::Shape::line_segment([center, minute_coord], Stroke::new(2.0, color));

    // The second hand
    let frac_secs = dt.second() as f32 + dt.nanosecond() as f32 / 1_000_000_000.;
    let second_angle = angle(60., frac_secs as f32);
    let second_coord = coord(second_angle, radius * 0.85);
    let second_hand =
        epaint::Shape::line_segment([center, second_coord], Stroke::new(2.0, Color32::RED));

    // Return vector of shapes
    vec![hour_hand, minute_hand, second_hand]
}
