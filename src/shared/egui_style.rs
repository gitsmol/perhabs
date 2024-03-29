use egui::epaint::Shadow;
use egui::style::{Selection, WidgetVisuals, Widgets};
use egui::{Color32, Rounding, Stroke, Visuals};

/// Custom dark theme.
pub fn dark_visuals() -> Visuals {
    let shadow = Shadow {
        extrusion: 3.0,
        color: Color32::from_black_alpha(96),
    };

    Visuals {
        dark_mode: true,
        override_text_color: None,
        widgets: dark_widgets(),
        selection: Selection::default(),
        hyperlink_color: Color32::from_rgb(90, 170, 255),
        faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
        extreme_bg_color: Color32::from_gray(10),            // e.g. TextEdit background
        code_bg_color: Color32::from_gray(64),
        warn_fg_color: Color32::from_rgb(255, 143, 0), // orange
        error_fg_color: Color32::from_rgb(255, 0, 0),  // red

        window_rounding: Rounding::same(2.0),
        window_shadow: shadow,
        window_fill: Color32::from_gray(27),
        window_stroke: Stroke::new(1.0, Color32::from_gray(60)),

        menu_rounding: Rounding::same(6.0),

        panel_fill: Color32::from_gray(27),

        popup_shadow: shadow,
        resize_corner_size: 12.0,
        // text_cursor_width: 2.0,
        text_cursor_preview: false,
        clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
        button_frame: true,
        collapsing_header_frame: false,
        indent_has_left_vline: true,

        striped: false,

        slider_trailing_fill: false,

        ..Default::default()
    }
}

/// Custom light theme.
pub fn light_visuals() -> Visuals {
    let shadow = Shadow {
        extrusion: 3.0,
        color: Color32::from_black_alpha(25),
    };

    Visuals {
        dark_mode: false,
        widgets: light_widgets(),
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

        window_shadow: shadow,
        window_fill: Color32::from_gray(248),
        window_stroke: Stroke::new(1.0, Color32::from_gray(190)),

        panel_fill: Color32::from_gray(250),

        popup_shadow: shadow,
        ..dark_visuals()
    }
}

/// Custom dark widget colors
pub fn dark_widgets() -> Widgets {
    let rounding = Rounding::same(1.0);

    Widgets {
        noninteractive: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(27),
            bg_fill: Color32::from_gray(27),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines
            fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
            rounding,
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(60), // button background
            bg_fill: Color32::from_gray(60),      // checkbox background
            bg_stroke: Default::default(),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
            rounding,
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(70),
            bg_fill: Color32::from_gray(70),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
            fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
            rounding: Rounding::same(2.0),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(55),
            bg_fill: Color32::from_gray(55),
            bg_stroke: Stroke::new(1.0, Color32::WHITE),
            fg_stroke: Stroke::new(2.0, Color32::WHITE),
            rounding,
            expansion: 1.0,
        },
        open: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(27),
            bg_fill: Color32::from_gray(27),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
            rounding,
            expansion: 0.0,
        },
    }
}

/// Custom light widget colors
pub fn light_widgets() -> Widgets {
    let rounding = Rounding::same(1.0);

    Widgets {
        noninteractive: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(248),
            bg_fill: Color32::from_gray(248),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(190)), // separators, indentation lines
            fg_stroke: Stroke::new(1.0, Color32::from_gray(80)),  // normal text color
            rounding,
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(230), // button background
            bg_fill: Color32::from_gray(230),      // checkbox background
            bg_stroke: Default::default(),
            fg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // button text
            rounding,
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(220),
            bg_fill: Color32::from_gray(220),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(105)), // e.g. hover over window edge or button
            fg_stroke: Stroke::new(1.5, Color32::BLACK),
            rounding: Rounding::same(2.0), // slightly more rounding on hover
            expansion: 1.0,
        },
        active: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(165),
            bg_fill: Color32::from_gray(165),
            bg_stroke: Stroke::new(1.0, Color32::BLACK),
            fg_stroke: Stroke::new(2.0, Color32::BLACK),
            rounding,

            expansion: 1.0,
        },
        open: WidgetVisuals {
            weak_bg_fill: Color32::from_gray(220),
            bg_fill: Color32::from_gray(220),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(160)),
            fg_stroke: Stroke::new(1.0, Color32::BLACK),
            rounding,

            expansion: 0.0,
        },
    }
}
