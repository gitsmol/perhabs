use egui::{vec2, Align, Align2, ScrollArea, Vec2};

use log::debug;
use perhabs::{
    shared::asset_loader::{
        exercise_config_collection::ExerciseConfigCollection, perhabs_config::PerhabsConfig,
        AssetSource,
    },
    shared::{self, egui_style, AppData},
    widgets,
    wm::SessionManager,
    wm::Windows,
};

pub struct Perhabs {
    tools: Windows,
    sessionman: SessionManager,
    appdata: AppData,
    show_about: bool,
    tts: tts::Tts,
    error: Option<String>,
}

impl Default for Perhabs {
    fn default() -> Self {
        Self {
            tools: Windows::default(),
            sessionman: SessionManager::default(),
            appdata: AppData::default(),
            show_about: false,
            error: None,

            #[cfg(target_os = "macos")]
            tts: tts::Tts::new(tts::Backends::AppKit).unwrap(), // NOTE default is AvKit which is bugged(?)
            #[cfg(not(target_os = "macos"))]
            tts: tts::Tts::default().unwrap(),
        }
    }
}

// ***********
// Internals
// ***********

impl Perhabs {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Use custom styles
        cc.egui_ctx.set_visuals(egui_style::light_visuals());

        Default::default()
    }

    /// Only returns true when both PerhabsConfig and ExcerciseConfig are present.
    fn guarantee_configs(&mut self) -> bool {
        if !self.guarantee_phconfig() {
            return false;
        };

        if !self.guarantee_excconfig() {
            return false;
        };

        // only return true if the guard clauses aren't triggered.
        true
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
                Err(e) => {
                    self.appdata.debug_messages.push(format!(
                        "AppData - Failed to get perhabsconfig from disk: {}, Trying to get config from web.",
                        e
                    ));
                    self.appdata.config_promise = Some(PerhabsConfig::from_web())
                }
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
                        Err(e) => {
                            self.appdata.debug_messages.push(format!(
                                "AppData - Failed to deserialize perhabsconfig: {}",
                                e
                            ));
                            Some(PerhabsConfig::default())
                        }
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
            None => panic!("ExcConfig - Fatal error: no PerhabsConfig found."),
        };

        // Is there a promise for a web download of a config?
        match &self.appdata.excconfig_promise {
            // No: try to get a config from disk.
            // If that fails, put a promise in place for the next loop of this function
            // If that fails, create a config from the hardcoded defaults.
            None => match ExerciseConfigCollection::from_disk(&format!(
                "{}{}",
                &config.disk_root, &config.excconfig_path,
            )) {
                Ok(mut res) => {
                    res.source = AssetSource::Disk;
                    self.appdata.excconfig = Some(res)
                }
                Err(e) => {
                    let path = format!("{}{}", &config.web_root, &config.excconfig_path);
                    self.appdata.excconfig_promise =
                        Some(ExerciseConfigCollection::from_web(&path));

                    debug!("No exercise config found on disk. Getting web config.");
                    let errormsg = format!(
                        "ExcConfig - No exercise config found on disk: {e}.\nGetting web config from {path}."
                    );
                    self.appdata.debug_messages.push(String::from(errormsg));
                }
            },
            // Yes: we have a promise.
            Some(promise) => {
                // Is the promise succesfully fulfilled?
                if let Some(Ok(resource)) = promise.ready() {
                    debug!("Promise for exercise config is ready.");
                    // Deserialize the data we got from the promise
                    let config =
                        serde_json::from_str::<ExerciseConfigCollection>(resource.text().unwrap());

                    // Store data in config, depending on successful deserialization.
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
                            let errormsg = format!(
                                "ExcConfig - Failed to deserialize exercise config: {}",
                                error
                            );
                            self.appdata.debug_messages.push(String::from(errormsg));
                            Some(ExerciseConfigCollection::default())
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

// ***********
// UI
// ***********

impl eframe::App for Perhabs {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Show errors if there are any
        self.error_window(ctx);

        // Persistent menubar at the top of the screen.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu_bar(ui, frame));

        // The central panel is where we display all windows and exercises.
        // Note the early return pattern in this code!
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show a loading screen until we have configs. Then show utility windows and session.
            if self.guarantee_configs() == false {
                widgets::loading_screen(ui);
                return;
            }

            // Show about window
            self.about_screen(ctx);

            // Always show single windows
            self.tools.windows(ctx, &self.appdata, &mut self.tts);

            // Show the session menu or an active session if present
            if self.sessionman.open.is_some() {
                self.sessionman
                    .session_show(ctx, &self.appdata, &mut self.tts);
                return;
            }

            // If there is no open session, show the exercise menu
            // First, determine if we are on a small screen or not
            let menu_width = {
                if ui.available_width() < 600. {
                    ui.available_width()
                } else {
                    600.
                }
            };

            if menu_width < 600. {
                self.menu_smallscreen(ctx, ui)
            } else {
                self.menu(ctx, ui);
            };
        });
    }
}

impl Perhabs {
    // Error window
    fn error_window(&mut self, ctx: &egui::Context) {
        // Check for error messages
        match self.appdata.error_rx.try_recv() {
            Ok(error) => {
                self.error = Some(error);
            }
            Err(_) => return,
        }

        let desired_size = {
            let avail_size = ctx.available_rect().size();
            if avail_size.x < 600.0 {
                avail_size
            } else {
                vec2(500.0, avail_size.y)
            }
        };
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .min_width(desired_size.x)
            .anchor(Align2::CENTER_TOP, vec2(0., desired_size.y * 0.2))
            .show(ctx, |ui| {
                if let Some(error) = &self.error {
                    ui.label(format!("{}", error));
                }
                if ui.button("Close").clicked() {
                    self.error = None
                }
            });
    }

    /// Persistent menu bar at the top of the screen
    fn menu_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                // Toggle dark mode
                widgets::dark_mode_toggle_button(ui);

                // Quit button
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("\u{2386} Quit").clicked() {
                    frame.close();
                };
            });

            // Tools menu
            ui.menu_button("Tools", |ui| {
                // Debug checkbox
                ui.checkbox(&mut self.appdata.debug, "Debug");
                // Available windows
                self.tools.labels(ui);
            });

            // About button
            ui.toggle_value(&mut self.show_about, "About");

            // Only show quit when a session is active.
            if let Some(session_name) = self.sessionman.open {
                ui.add_space(ui.available_width() - 85.);
                if ui.button("\u{2386} Quit session").clicked() {
                    self.sessionman.open = None;
                    // Reset the session on close
                    for session in &mut self.sessionman.sessions {
                        if session.name() == session_name {
                            session.reset();
                        }
                    }
                }
            }
        });
    }

    /// The exercise menu in its 2 column layout
    fn menu(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let spacer = ui.available_size() / 20.;

        egui::Window::new("Exercise menu")
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 2.0 * spacer.y),
            )
            .fixed_size(vec2(600., 400.))
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                self.sessionman.buttons_cols(ui);
            });
    }

    /// The exercise menu in its single column layout for small screens
    fn menu_smallscreen(&mut self, _: &egui::Context, ui: &mut egui::Ui) {
        ui.label("Perhabs consists of a number of exercises targeting different skills.\n\nThe menu at the top of the screen provides some tools.");
        ui.add_space(10.);
        ScrollArea::new([false, true])
            .drag_to_scroll(true)
            .show(ui, |ui| self.sessionman.buttons(ui));
    }

    fn about_screen(&mut self, ctx: &egui::Context) {
        let desired_size = {
            let avail_size = ctx.available_rect().size();
            if avail_size.x < 600.0 {
                avail_size
            } else {
                vec2(500.0, avail_size.y)
            }
        };
        egui::Window::new("About")
            .open(&mut self.show_about)
            .collapsible(false)
            .resizable(false)
            .min_width(desired_size.x)
            .anchor(Align2::CENTER_TOP, vec2(0., desired_size.y * 0.2))
            .show(ctx, |ui| {
                shared::about_screen(ui);
            });
    }
}
