use crate::exercises::Direction;
use egui::{
    emath::{RectTransform, Rot2},
    pos2, vec2,
    widget_text::WidgetTextGalley,
    Color32, Mesh, Pos2, Rect, Response, Sense, Shape, Stroke, TextStyle, WidgetText,
};

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

/// Large menu button
pub fn menu_button(
    ui: &mut egui::Ui,
    label_source: &str,
    description_source: &str,
) -> egui::Response {
    // 1. Deciding widget size:
    let desired_size = egui::vec2(ui.available_width(), 100.);
    let text_size = desired_size[0] * 0.8;

    // Some type conversions. Probably very inefficient; should fix. TODO!
    let label_wt: WidgetText = label_source.into();
    let label_galley = label_wt.into_galley(ui, Some(true), text_size, TextStyle::Body);
    let description_wt: WidgetText = description_source.into();
    let description_galley: WidgetTextGalley =
        description_wt.into_galley(ui, Some(true), text_size, TextStyle::Body);

    // 2. Allocating space:
    // This is where we get a region of the screen assigned.
    // We also tell the Ui to sense clicks in the allocated region.
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // 3. Interact: Time to check for clicks!
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
        let label_pos = ui
            .layout()
            .align_size_within_rect(vec2(50., 50.), rect)
            .center_top();
        // The description goes in the same position but a little lower.
        let description_pos = label_pos + vec2(0., 30.);
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
