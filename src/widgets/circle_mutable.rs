use std::f32::consts::TAU;

use chrono::Duration;
use egui::*;
use num::{Integer, ToPrimitive};

use crate::modules::time_formatting;

/// Circle to display and alter a duration.
/// Senses both clicks and drags.
/// When dragged, changes the given integer by another integer (change_by).
/// Performs bounds checking: won't change value beyond given min or max.
///
/// Works by dividing a circle into 100 points and plotting points along this circle
/// until reaching the remaining percentage. These points are connected to the center of
/// the circle. That way this second circle forms a convex polygon obscuring part of
/// the first circle to indicate a percentage that isn't 'done'.
pub fn circle_input_integer<T: Integer + ToPrimitive + Copy>(
    ui: &mut egui::Ui,
    value: &mut T,
    change_by: &T,
    min: &T,
    max: &T,
    label: &String,
    size: f32,
    stroke_color: Color32,
) -> Response {
    //
    // Basics
    //
    // Allocate space
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());

    // Only do all this if widget is visible
    if ui.is_rect_visible(rect) {
        // Set up some basics
        let radius = rect.height().min(rect.width()) * 0.4;
        let stroke_width = radius * 0.1;
        let painter = ui.painter();
        // Paint circle
        painter.circle(
            rect.center(),
            radius,
            Color32::TRANSPARENT,
            Stroke::new(stroke_width, stroke_color),
        );

        //
        // Paint obscured part of circle
        //
        let angle = |period, time: f32| TAU * (time.rem_euclid(period) / period) as f32 - TAU / 4.0;
        let coord = |angle: f32, radius: f32| {
            pos2(
                rect.center().x + radius * angle.cos(),
                rect.center().y + radius * angle.sin(),
            )
        };
        // Calculate duration as a percentage of min-max
        // There is some awkard type conversion involved here for the benefit
        // of this function being 'generic' for the trait Integer.
        let percentage_remaining = {
            if let Some(range) = (*max - *min).to_f32() {
                if let Some(mut float) = value.to_f32() {
                    if float > range {
                        float = range
                    }
                    100 - ((float / range as f32) * 100.0) as usize
                } else {
                    0
                }
            } else {
                0
            }
        };

        // Plot points to indicate remaining percentage and paint
        if percentage_remaining > 0 {
            let mut points = vec![];
            for j in 0..percentage_remaining {
                points.push(coord(angle(99., 99. - j as f32), radius + stroke_width));
            }
            points.push(rect.center());
            let bg_fill = ui.visuals().window_fill;
            painter.add(Shape::convex_polygon(points, bg_fill, Stroke::NONE));
        }

        // Paint inner circle
        painter.circle(
            rect.center(),
            radius - stroke_width / 2.,
            Color32::TRANSPARENT,
            ui.visuals().widgets.noninteractive.fg_stroke,
        );

        //
        // Paint text
        //
        // Create galley for data
        let value_string = {
            if let Some(int) = value.to_usize() {
                String::from(format!("{}", int))
            } else {
                String::from("Num err")
            }
        };
        let data_wt: WidgetText = value_string.into();
        let data_galley = data_wt.into_galley(ui, None, rect.width(), TextStyle::Heading);
        let data_galley_size = data_galley.size();

        // Create galley for label
        let label_wt: WidgetText = label.into();
        let label_galley = label_wt.into_galley(ui, None, rect.width(), TextStyle::Small);
        let label_galley_size = label_galley.size();

        // Paint galleys
        data_galley.paint_with_visuals(
            ui.painter(),
            rect.center() - (data_galley_size * 0.5),
            ui.style().noninteractive(),
        );
        label_galley.paint_with_visuals(
            ui.painter(),
            rect.center() - (label_galley_size * 0.5 - vec2(0., radius * 0.5)),
            ui.style().noninteractive(),
        );
        //
        // Handle changing the duration
        //
        let delta = response.drag_delta();
        let in_bounds = |i: &T| {
            if i > max || i < min {
                false
            } else {
                true
            }
        };
        match delta.y {
            d if d < -1.0 => {
                let new_value = *value + *change_by;
                if in_bounds(&new_value) {
                    *value = new_value
                }
            }
            d if d > 1.0 => {
                let new_value = *value - *change_by;
                if in_bounds(&new_value) {
                    *value = new_value
                }
            }
            _ => (),
        };
    }

    // return response
    response
}

/// Circle to display and alter a duration.
/// Senses both clicks and drags.
/// When dragged, changes the given duration by a given duration (change_by).
/// Performs bounds checking: won't change duration beyond given min or max.
///
/// Works by dividing a circle into 100 points and plotting points along this circle
/// until reaching the remaining percentage. These points are connected to the center of
/// the circle. That way this second circle forms a convex polygon obscuring part of
/// the first circle to indicate a percentage that isn't 'done'.
pub fn circle_input_duration(
    ui: &mut egui::Ui,
    duration: &mut Duration,
    change_by: &Duration,
    min: &Duration,
    max: &Duration,
    label: &String,
    size: f32,
    stroke_color: Color32,
) -> Response {
    //
    // Basics
    //
    // Set up positioning etc
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    let painter = ui.painter();
    let radius = rect.height().min(rect.width()) * 0.4;
    let stroke_width = radius * 0.1;

    // Only do all this if widget is visible
    if ui.is_rect_visible(rect) {
        // Paint circle
        painter.circle(
            rect.center(),
            radius,
            Color32::TRANSPARENT,
            Stroke::new(stroke_width, stroke_color),
        );

        //
        // Paint obscured part of circle
        //
        let angle = |period, time: f32| TAU * (time.rem_euclid(period) / period) as f32 - TAU / 4.0;
        let coord = |angle: f32, radius: f32| {
            pos2(
                rect.center().x + radius * angle.cos(),
                rect.center().y + radius * angle.sin(),
            )
        };
        // Calculate duration as a percentage of min-max
        let range = max.num_seconds() - min.num_seconds();
        let percentage_remaining = {
            let mut duration_secs = duration.num_seconds();
            if duration.num_seconds() > range {
                duration_secs = range
            }
            100 - ((duration_secs as f32 / range as f32) * 100.0) as usize
        };

        // Plot points to indicate remaining percentage and paint
        if percentage_remaining > 0 {
            let mut points = vec![];
            for j in 0..percentage_remaining {
                points.push(coord(angle(99., 99. - j as f32), radius + stroke_width));
            }
            points.push(rect.center());
            let bg_fill = ui.visuals().window_fill;
            painter.add(Shape::convex_polygon(points, bg_fill, Stroke::NONE));
        }

        // Paint inner circle
        painter.circle(
            rect.center(),
            radius - stroke_width / 2.,
            Color32::TRANSPARENT,
            ui.visuals().widgets.noninteractive.fg_stroke,
        );

        //
        // Paint text
        //
        // Paint data
        let data = time_formatting::format_min_secs(duration);
        let data_wt: WidgetText = data.into();
        let data_galley = data_wt.into_galley(ui, None, rect.width(), TextStyle::Heading);
        let data_galley_size = data_galley.size();
        data_galley.paint_with_visuals(
            ui.painter(),
            rect.center() - (data_galley_size * 0.5),
            ui.style().noninteractive(),
        );

        // Paint label
        let label_wt: WidgetText = label.into();
        let label_galley = label_wt.into_galley(ui, None, rect.width(), TextStyle::Small);
        let label_galley_size = label_galley.size();
        label_galley.paint_with_visuals(
            ui.painter(),
            rect.center() - (label_galley_size * 0.5 - vec2(0., radius * 0.5)),
            ui.style().noninteractive(),
        );

        //
        // Handle changing the duration
        //
        let delta = response.drag_delta();
        let in_bounds = |i: &Duration| {
            if i > max || i < min {
                false
            } else {
                true
            }
        };
        match delta.y {
            d if d < -1.0 => {
                if let Some(new_duration) = duration.checked_add(change_by) {
                    if in_bounds(&new_duration) {
                        *duration = new_duration
                    }
                }
            }
            d if d > 1.0 => {
                if let Some(new_duration) = duration.checked_sub(change_by) {
                    if in_bounds(&new_duration) {
                        *duration = new_duration
                    }
                }
            }
            _ => (),
        };
    }
    // return response
    response
}
