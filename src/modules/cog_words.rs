use crate::asset_loader::{self, AppData, SentenceFile};
use crate::windowman::{AppWin, View};
use egui::RichText;
use ehttp::{Response, Result};
use poll_promise::Promise;
use rand::prelude::*;
use tts::{self, Tts};

// The sentences and all config go here
struct Sentences {
    promise: Option<Promise<Result<Response>>>,
    selected_file: Option<SentenceFile>,
    contents: Option<Vec<String>>,
}

impl Default for Sentences {
    fn default() -> Self {
        Self {
            promise: None,
            selected_file: None,
            contents: None,
        }
    }
}

impl Sentences {
    /// Shuffle the file contents vec using the Fisher-Yates shuffle algorithm.
    fn shuffle_contents(&mut self) {
        if let Some(contents) = &mut self.contents {
            let length = contents.len();
            let mut rng = thread_rng();
            for i in 0..length {
                let j = rng.gen_range(i..length);
                let tmp = contents[i].clone();
                contents[i] = contents[j].clone();
                contents[j] = tmp;
            }
        }
    }
}

struct Session {
    active: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self { active: false }
    }
}

struct Answers {
    sequence: String,
    sequence_alpha: String,
    sequence_alpha_rev: String,
    sequence_rev: String,
}
impl Default for Answers {
    fn default() -> Self {
        Self {
            sequence: String::from("No sequence."),
            sequence_alpha: String::new(),
            sequence_alpha_rev: String::new(),
            sequence_rev: String::new(),
        }
    }
}

/// Sequences
pub struct CogWords {
    sentences: Sentences,

    session: Session,
    answers: Answers,
}

impl Default for CogWords {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            sentences: Sentences::default(),
            session: Session::default(),
        }
    }
}

impl CogWords {
    fn say(&mut self, spk: &mut tts::Tts) -> () {
        match spk.speak(&self.answers.sequence, false) {
            Ok(_) => debug!("TTS: Sentence spoken."),
            Err(e) => warn!("TTS error: {:?}", e),
        };
    }

    /// Pop the last sentence from the vec of file contents.
    fn pick_sentence(&mut self) {
        if let Some(contents) = &mut self.sentences.contents {
            // Get the last sentence in the vec
            match contents.pop() {
                Some(answer) => {
                    let answer = answer.to_lowercase();
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
                    self.session.active = false;
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
                    let diskpath = format!("{}{}", config.sentences_path_disk, file.filename);
                    // Try to load contents of selected file from disk
                    match asset_loader::get_sentences_disk(diskpath) {
                        // Found contents: store in self and shuffle
                        Ok(file) => {
                            self.sentences.contents = Some(file);
                            self.sentences.shuffle_contents()
                        }
                        // Can't load from disk: create a promise to load from web
                        Err(_) => {
                            let webpath = format!("{}{}", config.sentences_path_web, file.filename);
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

impl AppWin for CogWords {
    fn name(&self) -> &'static str {
        "Cog Words"
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, appdata: &AppData, tts: &mut Tts) {
        if !self.session.active {
            egui::Window::new(self.name())
                .default_size((250.0, 250.0))
                .vscroll(false)
                .resizable(false)
                .show(ctx, |ui| self.ui(ui, appdata, tts));
        }

        if self.session.active {
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.pick_sentence();
                self.say(tts);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.say(tts);
            }
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
    }
}

impl View for CogWords {
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // Show file picker.
        egui::ComboBox::from_label("Select sentences file")
            .selected_text(match &self.sentences.selected_file {
                Some(file) => file.language.clone(),
                None => String::from("No language selected."),
            })
            .show_ui(ui, |ui| {
                if let Some(config) = &appdata.config {
                    for file in &config.sentences_files {
                        ui.selectable_value(
                            &mut self.sentences.selected_file,
                            Some(file.to_owned()),
                            &file.language,
                        );
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
                    self.session.active = true;
                }
            }
        }
    }
    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, tts: &mut Tts) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.session = Session::default();
            };
        });

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 4.);

            ui.label("Sentence");
            ui.heading(RichText::new(&self.answers.sequence).size(25.));
            ui.add_space(20.);

            ui.label("Reversed");
            ui.label(RichText::new(&self.answers.sequence_rev).size(25.));
            ui.add_space(20.);

            ui.label("Alphabetical");
            ui.label(RichText::new(&self.answers.sequence_alpha).size(25.));
            ui.add_space(20.);

            ui.label("Alphabetical reversed");
            ui.label(RichText::new(&self.answers.sequence_alpha_rev).size(25.));
            ui.add_space(20.);

            ui.add_space(50.);
            ui.horizontal(|ui| {
                if ui.button("Repeat").clicked() {
                    self.say(tts)
                }
                if ui.button("Next").clicked() {
                    self.pick_sentence();
                    self.say(tts);
                }
            });
            ui.label("Press space for next sequence. Press return to repeat sequence.");
        });
    }
}
