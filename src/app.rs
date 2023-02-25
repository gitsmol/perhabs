use crate::windowman::Windows;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Perhabs {
    #[serde(skip)]
    windows: Windows,
    #[serde(skip)]
    speaker: tts::Tts,
}

impl Default for Perhabs {
    fn default() -> Self {
        Self {
            windows: Windows::default(),

            #[cfg(target_os = "macos")]
            speaker: tts::Tts::new(tts::Backends::AppKit).unwrap(), // NOTE default is AvKit which is bugged(?)
            #[cfg(not(target_os = "macos"))]
            speaker: tts::Tts::default().unwrap(),
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
}

impl eframe::App for Perhabs {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.label("Theme");
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.label(" | ");
                self.windows.labels(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {});
        // Show open windows
        self.windows.windows(ctx, &mut self.speaker);
    }
}
