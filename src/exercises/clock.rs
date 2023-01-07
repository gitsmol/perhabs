use crate::windowman::{AppWin, View};
use chrono::{Local, TimeZone, Timelike};
use eframe::epaint::{self, CircleShape};
use egui::{emath, pos2, vec2, Color32, Frame, Pos2, Rect, Stroke};

use std::f32::consts::PI;

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

        let dt = Local::now();
        ui.label(format!("{}", dt.to_string()));

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            // let time = ui.input().time; // time since app start in millis

            let desired_size = ui.available_width() * vec2(1., 1.);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

            let circ_center = to_screen * pos2(0.5, 0.);
            let radius = 100.;
            let circleshape = CircleShape::stroke(circ_center, radius, Stroke::new(2., color));
            let circle = epaint::Shape::Circle(circleshape);
            ui.painter().add(circle);

            let hands = draw_hands(circ_center, radius, color);
            ui.painter().extend(hands);
        });
    }
}

fn draw_hands(center: Pos2, radius: f32, color: Color32) -> Vec<epaint::Shape> {
    let dt = Local::now();
    let hour_angle = (2. * PI / 24.) * dt.hour() as f32;
    let minute_angle = (2. * PI / 60.) * dt.minute() as f32;
    let second_angle = (2. * PI / 60.) * dt.second() as f32;
    let hour_coord = calc_polar_coord(hour_angle, radius * 0.8, center);
    let minute_coord = calc_polar_coord(minute_angle, 100., center);
    let second_coord = calc_polar_coord(second_angle, 100., center);
    let mut shapes = vec![];
    for coord in [hour_coord, minute_coord, second_coord] {
        let shape = epaint::Shape::line_segment([center, coord], Stroke::new(2.0, color));
        shapes.push(shape)
    }
    shapes
}

fn calc_polar_coord(angle: f32, radius: f32, center: Pos2) -> Pos2 {
    let x = center[0] + radius * angle.cos();
    let y = center[1] + radius * angle.sin();
    pos2(x, y)
}

fn calc_circle_points(points: u32, radius: f32, center: Pos2) -> Vec<Pos2> {
    debug!("circ_center is {:?}", center);
    let slice: f32 = 2. * PI / points as f32;
    let mut result = vec![];
    for i in 0..points {
        let angle = slice * i as f32;
        let new_x = center[0] + radius * angle.cos();
        let new_y = center[1] + radius * angle.sin();
        debug!("Point is {:?}", (new_x, new_y));
        result.push(pos2(new_x, new_y));
    }

    result
}
