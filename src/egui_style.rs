use egui::epaint::Shadow;
use egui::style::{Selection, Widgets};
use egui::{Color32, Rounding, Stroke, Visuals};

/// Custom dark theme.
pub fn dark_visuals() -> Visuals {
    Visuals {
        dark_mode: true,
        override_text_color: None,
        widgets: Widgets::default(),
        selection: Selection::default(),
        hyperlink_color: Color32::from_rgb(90, 170, 255),
        faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
        extreme_bg_color: Color32::from_gray(10),            // e.g. TextEdit background
        code_bg_color: Color32::from_gray(64),
        warn_fg_color: Color32::from_rgb(255, 143, 0), // orange
        error_fg_color: Color32::from_rgb(255, 0, 0),  // red

        window_rounding: Rounding::same(2.0),
        window_shadow: Shadow::small_dark(),
        window_fill: Color32::from_gray(27),
        window_stroke: Stroke::new(1.0, Color32::from_gray(60)),

        menu_rounding: Rounding::same(6.0),

        panel_fill: Color32::from_gray(27),

        popup_shadow: Shadow::small_dark(),
        resize_corner_size: 12.0,
        text_cursor_width: 2.0,
        text_cursor_preview: false,
        clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
        button_frame: true,
        collapsing_header_frame: false,
        indent_has_left_vline: true,

        striped: false,

        slider_trailing_fill: false,
    }
}

/// Default light theme.
pub fn light_visuals() -> Visuals {
    Visuals {
        dark_mode: false,
        widgets: Widgets::light(),
        selection: Selection {
            bg_fill: Color32::from_rgb(144, 209, 255),
            stroke: Stroke::new(1.0, Color32::from_rgb(0, 83, 125)),
        },
        hyperlink_color: Color32::from_rgb(0, 155, 255),
        faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
        extreme_bg_color: Color32::from_gray(255),           // e.g. TextEdit background
        code_bg_color: Color32::from_gray(230),
        warn_fg_color: Color32::from_rgb(255, 100, 0), // slightly orange red. it's difficult to find a warning color that pops on bright background.
        error_fg_color: Color32::from_rgb(255, 0, 0),  // red

        window_shadow: Shadow::small_light(),
        window_fill: Color32::from_gray(248),
        window_stroke: Stroke::new(1.0, Color32::from_gray(190)),

        panel_fill: Color32::from_gray(248),

        popup_shadow: Shadow::small_light(),
        ..dark_visuals()
    }
}
