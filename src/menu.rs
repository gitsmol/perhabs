use egui::{vec2, widget_text::WidgetTextGalley, TextStyle, WidgetText};

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
            .align_size_within_rect(vec2(55., 50.), rect)
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
