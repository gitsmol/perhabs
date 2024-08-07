use crate::shared::asset_loader::sentences::Sentences;
use crate::shared::{asset_loader, AppData};
use crate::widgets::{loading_bar_vertical, loading_screen, menu_button};
use crate::wm::{Exercise, ExerciseType};
use egui::{vec2, Align, Color32, RichText, TextEdit, Vec2};
use tts::{self, Tts};

/// Sequences
pub struct EpisodicMemory {
    prompts: Sentences,
    session: bool,
    answer: String,
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self {
            prompts: Sentences::default(),
            session: false,
            answer: String::new(),
        }
    }
}

impl EpisodicMemory {
    /// Pop the last sentence from the vec of file contents.
    fn next_question(&mut self) {
        self.answer.clear();

        if let Some(contents) = &mut self.prompts.contents {
            // Get the last sentence in the vec
            match contents.pop() {
                Some(answer) => {
                    // Insert the answer at the start of the vec,
                    // like putting it at the bottom of a deck of cards
                    contents.insert(0, answer)
                }
                None => {
                    self.session = false;
                }
            };
        };
    }

    fn contents_guarantee(&mut self, appdata: &AppData) -> bool {
        // If we have contents, give a guarantee
        if let Some(_) = self.prompts.contents {
            return true;
        };

        // If we don't have contents, we may have a promise for a web download
        match &self.prompts.promise {
            // No we don't have a promise
            None => {
                // We need a config to be present
                let file = match &self.prompts.selected_file {
                    Some(file) => file,
                    None => return false,
                };
                if let Some(config) = &appdata.config {
                    let diskpath = format!(
                        "{}{}{}",
                        config.disk_root, config.episodic_memory_path, file.filename
                    );
                    // Try to load contents of selected file from disk
                    match asset_loader::sentences::get_sentences_disk(diskpath) {
                        // Found contents: store in self and shuffle
                        Ok(file) => {
                            self.prompts.contents = Some(file);
                            self.prompts.shuffle_contents()
                        }
                        // Can't load from disk: create a promise to load from web
                        Err(_) => {
                            let webpath = format!(
                                "{}{}{}",
                                config.web_root, config.episodic_memory_path, file.filename
                            );
                            self.prompts.promise =
                                Some(asset_loader::sentences::get_sentences_web(webpath));
                        }
                    };
                } else {
                    panic!("No PerhabsConfig found!")
                };
            }
            // Yes, we have a promise
            Some(promise) => {
                // Is the promise succesfully fulfilled?
                if let Some(Ok(resource)) = promise.ready() {
                    // Deserialize the data we got from the promise
                    let contents = resource.text().unwrap();

                    // Store contents of sentences file
                    self.prompts.contents =
                        match asset_loader::sentences::read_sentences_promise(contents) {
                            Ok(res) => Some(res),
                            // If deserialization fails, store hardcoded defaults
                            Err(_) => Some(asset_loader::sentences::default_sentences()),
                        };

                    // Finally, shuffle the downloaded/default contents
                    self.prompts.shuffle_contents()
                }
            }
        }

        // return false if we can't return a guarantee
        false
    }
}

impl Exercise for EpisodicMemory {
    fn name(&self) -> &'static str {
        "Episodic Memory Questions"
    }

    fn description(&self) -> &'static str {
        "Challenge yourself to remember things you've seen, heard and done."
    }

    fn help(&self) -> &'static str {
        "This exercise will ask you different questions about things in your past. Try to recall as much as you can to answer the questions."
    }

    fn excercise_type(&self) -> Vec<ExerciseType> {
        vec![ExerciseType::Cognitive]
    }

    fn reset(&mut self) {
        *self = EpisodicMemory::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        if !self.session {
            egui::Window::new(self.name())
                .anchor(
                    egui::Align2([Align::Center, Align::TOP]),
                    Vec2::new(0., 100.),
                )
                .fixed_size(vec2(350., 300.))
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .show(ctx, |ui| self.ui(ui, appdata, tts));
        }

        if self.session {
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        ui.label(self.help());
        ui.separator();

        if let Some(config) = &appdata.config {
            for file in &config.episodic_memory_files {
                if menu_button(ui, None, None, file.language.as_str(), "").clicked() {
                    self.prompts.selected_file = Some(file.to_owned());
                    // If the selected value changes set the contents to none.
                    // This triggers the contents guarantee and fetches the appropriate file.
                    self.prompts.contents = None;
                    self.session = true;
                };
            }
        }
    }
    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // Loading screen if we are still loading data
        if self.contents_guarantee(appdata) == false {
            loading_screen(ui);
            return;
        }

        //
        // The Session UI
        //
        let spacer = ui.available_height() / 30.;

        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = false
            };
        });

        ui.vertical_centered(|ui| {
            ui.add_space(spacer * 4.);

            if let Some(contents) = &self.prompts.contents {
                if let Some(question) = contents.last() {
                    ui.heading(RichText::new(question).size(25.));
                    ui.add_space(spacer * 2.);

                    // Calculate the percentage score for the vert loading bar.
                    let calc_perc_score = |text: &String| {
                        let perc = text.len() as f32 / 200.;
                        if perc > 1.0 {
                            1.0
                        } else {
                            perc
                        }
                    };

                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() * 0.5 - 150.);
                        ui.add_sized(vec2(300., 100.), TextEdit::multiline(&mut self.answer));
                        loading_bar_vertical(
                            ui,
                            calc_perc_score(&self.answer),
                            Color32::LIGHT_BLUE,
                        );
                    });

                    ui.add_space(spacer);
                    ui.label("Write down as much as you can remember.");
                    ui.label(match self.answer.len() {
                        l if l < 100 => "Try to fill the progress bar by writing.",
                        l if l > 100 && l <= 150 => "Well done. Can you think of more details?",
                        l if l > 150 => "Great job!",
                        _ => "",
                    });

                    ui.add_space(spacer / 4.);
                    if ui
                        .add_sized(vec2(spacer * 4., spacer * 2.), egui::Button::new("Next"))
                        .clicked()
                    {
                        self.next_question();
                    };
                }
            }
        });
    }
}
