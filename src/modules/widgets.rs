use crate::exercises::Direction;
use chrono::Duration;
use egui::{
    emath::{RectTransform, Rot2},
    pos2, vec2,
    widget_text::WidgetTextGalley,
    Color32, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, TextStyle, Vec2, WidgetText,
};
use num::{Integer, ToPrimitive};
use std::{f32::consts::TAU, iter::zip};

use super::time_formatting;

pub fn loading_screen(ui: &mut egui::Ui) {
    // Show loading screen while waiting for contents of file
    ui.horizontal(|ui| {
        ui.label("Loading file...");
        ui.spinner();
    });
}

pub fn loading(ui: &mut egui::Ui) {
    ui.horizontal_centered(|ui| {
        ui.heading("Loading...");
        ui.spinner();
    });
}

/// Vertical loading bar, quite narrow.
pub fn loading_bar_vertical(ui: &mut egui::Ui, progress: f32, fill: Color32) -> Response {
    let desired_size = vec2(ui.spacing().item_spacing.x, ui.available_height());
    let (outer_rect, response) =
        ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());
    let painter = ui.painter();
    let bg_fill = ui.style().visuals.widgets.active.bg_fill;
    let rounding = outer_rect.height() * 0.7;

    painter.rect(outer_rect, rounding, bg_fill, Stroke::NONE);

    let inner_height = outer_rect.size().y * progress;
    let inner_rect = Rect::from_min_size(
        outer_rect.left_bottom() - vec2(0., inner_height),
        vec2(outer_rect.size().x, inner_height),
    );

    painter.rect(
        inner_rect,
        rounding,
        fill,
        Stroke::new(1.0, fill.gamma_multiply(1.2)),
    );

    response
}

/// Circle to present a number
pub fn circle_with_data(
    ui: &mut egui::Ui,
    data: &String,
    label: &String,
    size: f32,
    stroke_color: Color32,
) {
    // Set up positioning etc
    let (_, rect) = ui.allocate_space(vec2(size, size));
    let painter = ui.painter();
    let radius = rect.height().min(rect.width()) * 0.4;
    let stroke_width = radius * 0.1;

    // Paint circle
    painter.circle(
        rect.center(),
        radius,
        Color32::TRANSPARENT,
        Stroke::new(stroke_width, stroke_color),
    );

    // Paint data
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
}

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

/// A set of widgets to configure the evalation parameters.
pub fn eval_config_widgets(ui: &mut egui::Ui, duration: &mut Duration, reps: &mut usize) {
    ui.heading("Session length");
    ui.separator();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_size(vec2(225., 100.));
        ui.label("Pick a session length or manually change duration and repetitions. The session ends when time runs out or the maximum number of repetitions is reached.");
        ui.add_space(15.);
        ui.horizontal(|ui| {
            for (name, i) in zip(["Short", "Medium", "Long"], [60, 90, 120]) {
                if ui.add_sized(vec2(60., 30.), |ui: &mut egui::Ui| ui.button(name)
                ).clicked() {
                    *duration = Duration::seconds(i);
                    *reps = i as usize;
            };

            };
        });
        });
        ui.add_space(25.);
        ui.vertical(|ui| {

            ui.horizontal(|ui| {
                let duration_circle = circle_input_duration(
                    ui,
                    duration,
                    &Duration::seconds(1),
                    &Duration::seconds(10),
                    &Duration::seconds(120),
                    &String::from("Duration"),
                    125.,
                    Color32::DARK_BLUE,
                );
                if duration_circle.double_clicked() {
                    *duration = Duration::seconds(60);
                }
                let reps_circle = circle_input_integer(
                    ui,
                    reps,
                    &1,
                    &5,
                    &120,
                    &String::from("Repetitions"),
                    125.,
                    Color32::DARK_BLUE,
                );
                if reps_circle.double_clicked() {
                    *reps = 60;
                }
            });

        });
    });
    ui.separator();
}

/// Large menu button

/// ## Params
/// override_size:  When Some, determines the size of the button.
///                 When None, vec2(ui.available_width(), 100.) is used.
pub fn menu_button(
    ui: &mut egui::Ui,
    override_size: Option<Vec2>,
    label_source: &str,
    description_source: &str,
) -> egui::Response {
    // Determine sizes
    let desired_size = {
        if let Some(size) = override_size {
            size
        } else {
            egui::vec2(ui.available_width(), 100.)
        }
    };

    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // Prepare responses
    if response.clicked() {
        response.mark_changed(); // report back that the value changed
    }

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            format!("{}: {}", label_source, description_source),
        )
    });

    // Some type conversions and setting up text.
    let text_size = desired_size[0] * 0.8;
    let label_wt: WidgetText = label_source.into();
    let label_galley = label_wt.into_galley(ui, None, text_size, TextStyle::Body);
    let description_wt: WidgetText = description_source.into();
    let description_galley: WidgetTextGalley =
        description_wt.into_galley(ui, Some(true), text_size, TextStyle::Body);

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
        let rect = rect.expand(visuals.expansion);
        let radius = 0.1 * rect.height(); // Round corners.
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);

        // Put the label in the right position within the button.
        let margin = (rect.height() * 0.25).min(rect.width() * 0.25);
        let label_pos = rect.left_top() + vec2(margin, margin);
        // The description goes in the same position but a little lower.
        let description_pos = label_pos + vec2(0., margin);
        label_galley.paint_with_visuals(ui.painter(), label_pos, visuals);
        description_galley.paint_with_visuals(ui.painter(), description_pos, visuals);
    }

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, ...) and maybe show a tooltip:
    response
}

/// Return an arrow shaped Mesh suitable for egui::Painter.
pub fn arrow_shape(
    pos: Pos2,
    arrow_size: f32,
    direction: &Direction,
    to_screen: RectTransform,
    color: Color32,
) -> Shape {
    // Define some basic measures. M = measure, H = half measure.
    let m = arrow_size / 3. * 0.02;
    let h = m / 2.;

    // Create a mesh
    let mut arrow = Mesh::default();

    // Calculate arrowhead triangle positions and add to mesh.
    let right_arrow = vec![
        to_screen * pos2(pos.x + m, pos.y), // The tip
        to_screen * pos2(pos.x, pos.y + m), // Right
        to_screen * pos2(pos.x, pos.y - m), // Left
    ];
    for pos in right_arrow.iter() {
        arrow.colored_vertex(pos.to_owned(), color);
    }
    arrow.add_triangle(0, 1, 2);

    // Add the rectangular arrow tail.
    let tail_top_left = to_screen * (pos + vec2(-m, -h));
    let tail_bottom_right = to_screen * (pos + vec2(0., h));
    let r = Rect::from_two_pos(tail_top_left, tail_bottom_right);
    arrow.add_colored_rect(r, color);

    // Rotate the arrow in the right direction. The default points to the right.
    match direction {
        Direction::Up => arrow.rotate(Rot2::from_angle(-1.570796), to_screen * pos),
        Direction::Down => arrow.rotate(Rot2::from_angle(1.570796), to_screen * pos),
        Direction::Left => arrow.rotate(Rot2::from_angle(3.141593), to_screen * pos),
        Direction::Right => (),
    }

    Shape::Mesh(arrow)
}
