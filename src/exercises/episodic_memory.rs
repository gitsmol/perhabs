use crate::modules::asset_loader::{self, AppData};
use crate::modules::sentences::Sentences;
use crate::wm::sessionman::Exercise;
use egui::{vec2, Align, RichText, Vec2};
use tts::{self, Tts};

/// Sequences
pub struct EpisodicMemory {
    sentences: Sentences,
    session: bool,
    answer: String,
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self {
            sentences: Sentences::default(),
            session: false,
            answer: String::new(),
        }
    }
}

impl EpisodicMemory {
    fn say(&mut self, spk: &mut tts::Tts) -> () {
        if let Some(contents) = &self.sentences.contents {
            if let Some(question) = contents.last() {
                match spk.speak(question, false) {
                    Ok(_) => debug!("TTS: Sentence spoken."),
                    Err(e) => warn!("TTS error: {:?}", e),
                };
            }
        }
    }

    /// Pop the last sentence from the vec of file contents.
    fn next_question(&mut self) {
        self.answer.clear();

        if let Some(contents) = &mut self.sentences.contents {
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
        if let Some(_) = self.sentences.contents {
            return true;
        };

        // If we don't have contents, we may have a promise for a web download
        match &self.sentences.promise {
            // No we don't have a promise
            None => {
                // We need a config to be present
                let file = match &self.sentences.selected_file {
                    Some(file) => file,
                    None => return false,
                };
                if let Some(config) = &appdata.config {
                    let diskpath = format!("{}{}", config.episodic_memory_path_disk, file.filename);
                    // Try to load contents of selected file from disk
                    match asset_loader::get_sentences_disk(diskpath) {
                        // Found contents: store in self and shuffle
                        Ok(file) => {
                            self.sentences.contents = Some(file);
                            self.sentences.shuffle_contents()
                        }
                        // Can't load from disk: create a promise to load from web
                        Err(_) => {
                            let webpath =
                                format!("{}{}", config.episodic_memory_path_disk, file.filename);
                            self.sentences.promise = Some(asset_loader::get_sentences_web(webpath));
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
                    self.sentences.contents = match asset_loader::read_sentences_promise(contents) {
                        Ok(res) => Some(res),
                        // If deserialization fails, store hardcoded defaults
                        Err(_) => Some(asset_loader::default_sentences()),
                    };

                    // Finally, shuffle the downloaded/default contents
                    self.sentences.shuffle_contents()
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
            // if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            //     self.next_question();
            //     self.say(tts);
            // }
            // if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            //     self.say(tts);
            // }
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // Show file picker.
        egui::ComboBox::from_label("Select sentences file")
            .selected_text(match &self.sentences.selected_file {
                Some(file) => file.language.clone(),
                None => String::from("No language selected."),
            })
            .show_ui(ui, |ui| {
                if let Some(config) = &appdata.config {
                    for file in &config.episodic_memory_files {
                        if ui
                            .selectable_value(
                                &mut self.sentences.selected_file,
                                Some(file.to_owned()),
                                &file.language,
                            )
                            .changed()
                        {
                            // If the selected value changes set the contents to none.
                            // This triggers the contents guarantee and fetches the appropriate file.
                            self.sentences.contents = None;
                        };
                    }
                }
            });

        // Load contents of selected file
        if let Some(_) = self.sentences.selected_file {
            if self.contents_guarantee(appdata) == false {
                // Show loading screen while waiting for contents of file
                ui.horizontal(|ui| {
                    ui.label("Loading file...");
                    ui.spinner();
                });

                return;
            } else {
                // Show session button only when we have a file loaded.
                if ui.button("Start session").clicked() {
                    self.session = true;
                }
            }
        }
    }
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, tts: &mut Tts) {
        let spacer = ui.available_height() / 30.;

        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = false
            };
        });

        ui.vertical_centered(|ui| {
            ui.add_space(spacer * 4.);

            if let Some(contents) = &self.sentences.contents {
                if let Some(question) = contents.last() {
                    ui.heading(RichText::new(question).size(25.));
                    ui.add_space(spacer * 2.);
                    ui.text_edit_multiline(&mut self.answer);
                    ui.label("Write down as much as you can remember about the question.");
                    ui.label(match self.answer.len() {
                        l if l < 100 => "Try to write down some more.",
                        l if l > 100 => "Good job. Can you think of more details?",
                        l if l > 150 => "Excellent work!",
                        _ => "",
                    });

                    ui.add_space(spacer * 2.);
                    if ui
                        .add_sized(vec2(spacer * 4., spacer * 2.), egui::Button::new("Repeat"))
                        .clicked()
                    {
                        self.say(tts);
                    };

                    ui.add_space(spacer / 4.);
                    if ui
                        .add_sized(vec2(spacer * 4., spacer * 2.), egui::Button::new("Next"))
                        .clicked()
                    {
                        self.next_question();
                        self.say(tts);
                    };

                    ui.add_space(spacer);
                    ui.label("Press space for next sequence. Press return to repeat sequence.");
                }
            }
        });
    }
}
