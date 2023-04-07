use eframe::epaint::Shadow;
use egui::{Rounding, Style, Visuals};

use crate::{
    asset_loader::{self, AppData, AssetSource, ExcConfig, PerhabsConfig},
    sessionman::Sessions,
    windowman::Windows,
};

pub struct Perhabs {
    windows: Windows,
    sessions: Sessions,
    appdata: AppData,
    speaker: tts::Tts,
}

impl Default for Perhabs {
    fn default() -> Self {
        Self {
            windows: Windows::default(),
            sessions: Sessions::default(),
            appdata: AppData::default(),

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

        let mut visuals = Visuals::default();
        visuals.window_rounding = Rounding::same(2.);
        visuals.window_shadow = Shadow::small_light();
        cc.egui_ctx.set_visuals(visuals);

        Default::default()
    }

    fn guarantee_configs(&mut self) -> bool {
        // If we have a PerhabsConfig and ExcConfig, don't do anything.
        if self.guarantee_phconfig() == true {
            // NOTE we can't have ExcConfig without PerhabsConfig
            if self.guarantee_excconfig() == true {
                return true;
            }
        }

        // return false if we dont early return true
        false
    }

    /// Guarantees a configuration; returns false when there is no PerhabsConfig.
    fn guarantee_phconfig(&mut self) -> bool {
        if let Some(_) = &self.appdata.config {
            return true;
        }

        // Is there a promise for a web download of a config?
        match &self.appdata.config_promise {
            // No: try to get a config from disk.
            // If that fails, put a promise in place for the next loop of this function
            // If that fails, create a config from the hardcoded defaults.
            None => match PerhabsConfig::from_disk() {
                Ok(mut res) => {
                    res.source = AssetSource::Disk;
                    self.appdata.config = Some(res);
                }
                Err(_) => self.appdata.config_promise = Some(PerhabsConfig::from_web()),
            },
            // Yes: we have a promise.
            Some(promise) => {
                // Is the promise succesfully fulfilled?
                if let Some(Ok(resource)) = promise.ready() {
                    debug!("Promise for Perhabs config is ready.");
                    // Deserialize the data we got from the promise
                    let config = serde_json::from_str::<PerhabsConfig>(resource.text().unwrap());

                    // Store data in config, depending on the success deserialization.
                    // If deser fails, store hardcoded defaults.
                    self.appdata.config = match config {
                        Ok(mut res) => {
                            res.source = AssetSource::Web;
                            Some(res)
                        }
                        // If deserialization fails, store hardcoded defaults
                        Err(_) => Some(PerhabsConfig::default()),
                    }
                }
            }
        }

        // Eventually we return false so whatever requested this guarantee
        // knows there is no config (yet).
        false
    }

    /// Guarantees a configuration; returns false when there is no ExcConfig.
    fn guarantee_excconfig(&mut self) -> bool {
        // Return true if there is a ExcConfig in place.
        if let Some(_) = &self.appdata.excconfig {
            return true;
        }

        let config = match &self.appdata.config {
            Some(res) => res,
            None => panic!("Fatal error: no PerhabsConfig found."),
        };

        // Is there a promise for a web download of a config?
        match &self.appdata.excconfig_promise {
            // No: try to get a config from disk.
            // If that fails, put a promise in place for the next loop of this function
            // If that fails, create a config from the hardcoded defaults.
            None => match ExcConfig::from_disk(&config.excconfig_path_disk) {
                Ok(mut res) => {
                    res.source = AssetSource::Disk;
                    self.appdata.excconfig = Some(res)
                }
                Err(_) => {
                    debug!("No exercise config found on disk. Getting web config.");
                    let path = &config.excconfig_path_web;
                    self.appdata.excconfig_promise = Some(ExcConfig::from_web(path))
                }
            },
            // Yes: we have a promise.
            Some(promise) => {
                // Is the promise succesfully fulfilled?
                if let Some(Ok(resource)) = promise.ready() {
                    debug!("Promise for exercise config is ready.");
                    // Deserialize the data we got from the promise
                    let config = serde_json::from_str::<ExcConfig>(resource.text().unwrap());

                    // Store data in config, depending on the success deserialization.
                    // If deser fails, store hardcoded defaults.
                    self.appdata.excconfig = match config {
                        Ok(mut res) => {
                            debug!("Succesfully deserialized exercise config.");
                            res.source = AssetSource::Web;
                            Some(res)
                        }
                        // If deserialization fails, store hardcoded defaults
                        Err(error) => {
                            debug!("Failed to deserialize exercise config: {}", error);
                            Some(ExcConfig::default())
                        }
                    }
                }
            }
        }

        // Eventually we return false so whatever requested this guarantee
        // knows there is no config (yet).
        false
    }
}

impl eframe::App for Perhabs {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.label("Theme");
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.label(" | ");
                ui.label("Tools");
                self.windows.labels(ui);
            });
        });

        egui::Window::new("Test").show(ctx, |ui| self.sessions.buttons(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.guarantee_configs() == false {
                asset_loader::loading(ui);
            } else {
                self.windows.windows(ctx, &self.appdata, &mut self.speaker)
            }
        });

        self.sessions
            .sessions(ctx, &self.appdata, &mut self.speaker);
    }
}
