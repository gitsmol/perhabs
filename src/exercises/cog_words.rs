use crate::shared::asset_loader::sentences::{SentenceFile, Sentences};
use crate::shared::{asset_loader, AppData};
use crate::widgets::menu_button;
use crate::wm::{Exercise, ExerciseType};
use egui::{vec2, Align, RichText, Vec2};
use tts::{self, Tts};

struct Answers {
    sequence: String,
    sequence_alpha: String,
    sequence_alpha_rev: String,
    sequence_rev: String,
}
impl Default for Answers {
    fn default() -> Self {
        Self {
            sequence: String::from("Press continue to begin."),
            sequence_alpha: String::new(),
            sequence_alpha_rev: String::new(),
            sequence_rev: String::new(),
        }
    }
}

/// Sequences
pub struct CogWords {
    sentences: Sentences,
    answers: Answers,
    session_active: bool,
    display_answer: bool,
}

impl Default for CogWords {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            sentences: Sentences::default(),
            session_active: false,
            display_answer: true,
        }
    }
}

impl CogWords {
    fn next(&mut self, spk: &mut tts::Tts) {
        match self.display_answer {
            true => {
                // Hide answer, pick new sentence, speak the sentence
                self.display_answer = false;
                self.pick_sequence();
                self.say(spk);
            }
            false => {
                // Unhide answer
                self.display_answer = true;
            }
        }
    }

    fn say(&mut self, spk: &mut tts::Tts) {
        match spk.speak(&self.answers.sequence, false) {
            Ok(_) => debug!("TTS: Sentence spoken."),
            Err(e) => warn!("TTS error: {:?}", e),
        };
    }

    /// Pop the last sentence from the vec of file contents.
    fn pick_sequence(&mut self) {
        if let Some(contents) = &mut self.sentences.contents {
            // Get the last sentence in the vec
            match contents.pop() {
                Some(answer) => {
                    let answer = answer.to_lowercase();
                    // This is a relatively simple way of filtering out anything
                    // that isn't an alpha character or a space. Without this,
                    // the answers show interpunction in the wrong order.
                    let answer: String = answer
                        .chars()
                        .filter(|x| match x {
                            'A'..='Z' => true,
                            'a'..='z' => true,
                            ' ' => true,
                            _ => false,
                        })
                        .collect();
                    self.answers.sequence = answer.to_owned();
                    let mut sorted: Vec<&str> = answer.split(" ").collect();
                    sorted.reverse();
                    self.answers.sequence_rev = sorted.join(" ");
                    sorted.sort();
                    self.answers.sequence_alpha = sorted.join(" ");
                    sorted.reverse();
                    self.answers.sequence_alpha_rev = sorted.join(" ");

                    // Insert the answer at the start of the vec,
                    // like putting it at the bottom of a deck of cards
                    contents.insert(0, answer)
                }
                None => {
                    self.session_active = false;
                }
            };
        };
    }

    /// Returns a boolean to indicate if the exercise content is loaded.
    /// If no contents are found, works to retrieve them either from disk or web.
    fn contents_guarantee(&mut self, appdata: &AppData) -> bool {
        // If we have contents, give a positive guarantee
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
                    let diskpath = format!(
                        "{}{}{}",
                        config.disk_root, config.sentences_path, file.filename
                    );
                    // Try to load contents of selected file from disk
                    match asset_loader::sentences::get_sentences_disk(diskpath) {
                        // Found contents: store in self and shuffle
                        Ok(file) => {
                            self.sentences.contents = Some(file);
                            self.sentences.shuffle_contents()
                        }
                        // Can't load from disk: create a promise to load from web
                        Err(_) => {
                            let webpath = format!(
                                "{}{}{}",
                                config.web_root, config.sentences_path, file.filename
                            );
                            self.sentences.promise =
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
                    self.sentences.contents =
                        match asset_loader::sentences::read_sentences_promise(contents) {
                            Ok(res) => Some(res),
                            // If deserialization fails, store hardcoded defaults
                            Err(_) => Some(asset_loader::sentences::default_sentences()),
                        };

                    // Finally, shuffle the downloaded/default contents
                    self.sentences.shuffle_contents()
                }
            }
        }

        // return false if we can't return a guarantee
        false
    }

    /// Reads keypress to progress the exercise
    fn read_keypress(&mut self, ctx: &egui::Context, spk: &mut tts::Tts) {
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.next(spk);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.say(spk);
        }
    }

    /// A simple loading screen while we have no file contents loaded
    fn loading_screen(&mut self, ui: &mut egui::Ui, appdata: &AppData) {
        // If we have selected a file, try to load it
        // When it is loaded, start the session.
        if self.contents_guarantee(appdata) == false {
            if let Some(_) = self.sentences.selected_file {
                // Show loading screen while waiting for contents of file
                ui.horizontal(|ui| {
                    ui.label("Loading file...");
                    ui.spinner();
                });

                // Provide way to reset state if we somehow can't load a file
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.reset();
                    }
                });
            }
        }
    }
}

impl Exercise for CogWords {
    fn name(&self) -> &'static str {
        "Cognitive Words"
    }

    fn description(&self) -> &'static str {
        "Recall and reorder a sequence of words."
    }

    fn help(&self) -> &'static str {
        "This exercise uses your computers voice to say a random sentence out loud. It is up to you to reorder the words in this sentence.

Each sentence will be shown in full, alongside with
- the words reversed
- the words ordered alphabetically (A-Z)
- the words ordered alphabetically reversed (Z-A)

Pick your language and work your brain!"
    }

    fn excercise_type(&self) -> Vec<ExerciseType> {
        vec![ExerciseType::Cognitive]
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        let menu_window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(350., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        if !self.session_active {
            menu_window.show(ctx, |ui| self.ui(ui, appdata, tts));
        }

        if self.session_active {
            self.read_keypress(ctx, tts);
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // An explanation of this exercise.
        ui.label(self.help());
        ui.separator();

        // Show language picker
        // First define what happens when we click a language
        let mut func = |file: &SentenceFile| {
            // Select file
            self.sentences.selected_file = Some(file.to_owned());
            // Trigger content loading
            self.sentences.contents = None;
            // Activate session
            self.session_active = true;
        };

        if let Some(config) = &appdata.config {
            let buttons_total: f32 = config.sentences_files.len() as f32;
            let col_1_range = buttons_total - (buttons_total / 2.).floor();

            ui.columns(2, |col| {
                // Column 1 gets populated with at least half the buttons
                for i in 0..col_1_range as usize {
                    if let Some(file) = config.sentences_files.get(i) {
                        if menu_button(&mut col[0], None, None, file.language.as_str(), "")
                            .clicked()
                        {
                            func(file);
                        };
                    };
                }

                // Column 2 gets populated with the remaining buttons
                for i in col_1_range as usize..buttons_total as usize {
                    if let Some(file) = config.sentences_files.get(i) {
                        if menu_button(&mut col[1], None, None, file.language.as_str(), "")
                            .clicked()
                        {
                            func(file);
                        };
                    };
                }
            });
        }
    }

    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut Tts) {
        // Show loading screen if necessary
        self.loading_screen(ui, appdata);

        let spacer = ui.available_height() / 30.;

        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.reset()
            };
        });

        ui.vertical_centered(|ui| {
            if self.display_answer {
                ui.add_space(spacer * 4.);

                ui.label("Sentence");
                ui.heading(RichText::new(&self.answers.sequence).size(25.));
                ui.add_space(spacer);

                ui.label("Reversed");
                ui.label(RichText::new(&self.answers.sequence_rev).size(25.));
                ui.add_space(spacer);

                ui.label("Alphabetical");
                ui.label(RichText::new(&self.answers.sequence_alpha).size(25.));
                ui.add_space(spacer);

                ui.label("Alphabetical reversed");
                ui.label(RichText::new(&self.answers.sequence_alpha_rev).size(25.));
                ui.add_space(spacer);
            }

            if !self.display_answer {
                ui.add_space(spacer * 4.);
                ui.label("Try to reorder the words in your head.\nPress repeat (enter) to hear the sentence again.");
                ui.add_space(spacer * 9.);
            }

            ui.add_space(spacer * 2.);
            if ui
                .add_sized(vec2(spacer * 4., spacer * 2.), egui::Button::new("Repeat"))
                .clicked()
            {
                self.say(tts);
            };

            ui.add_space(spacer / 4.);
            if ui
                .add_sized(
                    vec2(spacer * 4., spacer * 2.),
                    egui::Button::new("Continue"),
                )
                .clicked()
            {
                self.next(tts);
            };

            ui.add_space(spacer);
            ui.label("Press space to continue. Press return to repeat sequence.");
        });
    }
}
