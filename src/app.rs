use egui::{Align2, Vec2};

use crate::windowman::Windows;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Perhabs {
    error_show: bool,
    error_msg: String,
    #[serde(skip)]
    windows: Windows,
    #[serde(skip)]
    speaker: tts::Tts,
}

impl Default for Perhabs {
    fn default() -> Self {
        Self {
            error_show: false,
            windows: Windows::default(),
            error_msg: "".to_string(),
            speaker: tts::Tts::new(tts::Backends::AppKit).unwrap(), // TODO use default (but fix Mac!)
        }
    }
}

impl Perhabs {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn error(&mut self, ctx: &egui::Context) -> () {
        egui::Window::new("Error").show(ctx, |ui| {
            ui.label(&self.error_msg);
            if ui.button("Close").clicked() {
                self.error_show = false;
                self.error_msg = "".to_string()
            };
        });
    }
}

impl eframe::App for Perhabs {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            error_show,
            error_msg,
            windows,
            speaker,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Window::new("Windows")
                .anchor(Align2::RIGHT_TOP, (0., 0.))
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Theme");
                        egui::widgets::global_dark_light_mode_buttons(ui);
                    });
                    ui.separator();
                    self.windows.checkboxes(ui);
                });
            // Show open windows
            self.windows.windows(ctx, &mut self.speaker);
        });

        if self.error_show {
            self.error(ctx);
        }
    }
}
