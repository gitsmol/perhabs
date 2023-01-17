use crate::windowman::{AppWin, View};
use fastrand;

use perhabs::{dirwalk, numvec_to_string, read_file};
use std::{
    io::BufRead,
    path::{Path, PathBuf},
    time::Duration,
};
use tts;

struct SourceFile {
    dirfiles: Vec<PathBuf>,
    sel_file_path: PathBuf,
    contents: Vec<String>,
}

#[derive(PartialEq, strum_macros::Display)]
enum ExerciseType {
    Numbers,
    Words,
}

struct ExcerciseConfig {
    keypress_delay: Duration,
    exercise_type: ExerciseType,
}

pub struct Sequences {
    seq_length: usize,
    seq_show: bool,
    sequence: String,
    sequence_alpha: String,
    sequence_alpha_rev: String,
    sequence_rev: String,
    file: SourceFile,
    config: ExcerciseConfig,
}

impl Default for Sequences {
    fn default() -> Self {
        let dirs = match dirwalk(Path::new("appdata")) {
            Ok(dirs) => dirs,
            Err(e) => {
                warn!("Can't find dir: {:?}", e);
                vec![]
            }
        };
        let sourcefile = SourceFile {
            dirfiles: dirs,
            sel_file_path: PathBuf::new(),
            contents: vec![],
        };

        Self {
            seq_length: 4,
            seq_show: false,
            sequence: String::from("No sequence."),
            sequence_alpha: String::new(),
            sequence_alpha_rev: String::new(),
            sequence_rev: String::new(),
            file: sourcefile,
            config: ExcerciseConfig {
                keypress_delay: Duration::from_secs(2),
                exercise_type: ExerciseType::Words,
            },
        }
    }
}

impl Sequences {
    fn say(&mut self, spk: &mut tts::Tts) -> () {
        match spk.speak(&self.sequence, false) {
            Ok(_) => debug!("TTS: Sentence spoken."),
            Err(e) => warn!("TTS error: {:?}", e),
        };
    }

    fn get_file(&mut self) -> () {
        self.file.contents.clear();
        let lines = read_file(&self.file.sel_file_path);
        for line in lines.lines() {
            if let Ok(ip) = line {
                self.file.contents.push(ip);
            }
        }
    }

    fn pick_next(&mut self) -> () {
        match self.config.exercise_type {
            ExerciseType::Words => self.pick_sentence(),
            ExerciseType::Numbers => self.pick_numbers(),
        };
    }
    fn pick_numbers(&mut self) -> () {
        let mut seq = vec![];
        while seq.len() < self.seq_length {
            // this means no seq longer than 11 numbers (0..10)!
            let num = fastrand::u32(0..=10);
            if !seq.contains(&num) {
                seq.push(num);
            };
        }

        self.sequence = numvec_to_string(&seq);
        seq.reverse();
        self.sequence_rev = numvec_to_string(&seq);
        seq.sort();
        self.sequence_alpha = numvec_to_string(&seq);
        seq.reverse();
        self.sequence_alpha_rev = numvec_to_string(&seq);
    }

    fn pick_sentence(&mut self) -> () {
        let max = self.file.contents.len();
        if max > 0 {
            let randnum = fastrand::usize(0..max);
            self.sequence = self.file.contents[randnum].clone().to_lowercase();
            let mut sorted: Vec<&str> = self.sequence.split(" ").collect();
            sorted.reverse();
            self.sequence_rev = sorted.join(" ");
            sorted.sort();
            self.sequence_alpha = sorted.join(" ");
            sorted.reverse();
            self.sequence_alpha_rev = sorted.join(" ");
        }
    }
    fn show_answer(&mut self, ui: &mut egui::Ui) {}
}

impl AppWin for Sequences {
    fn name(&self) -> &'static str {
        "Cog Sequences"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, mut spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_size((250.0, 250.0))
            .vscroll(false)
            .resizable(true)
            .show(ctx, |ui| self.ui(ui, &mut spk));
        if ctx.input().key_pressed(egui::Key::Space) {
            self.pick_next();
            self.say(spk);
        }
        if ctx.input().key_pressed(egui::Key::Enter) {
            self.say(spk);
        }
    }
}

impl View for Sequences {
    fn ui(&mut self, ui: &mut egui::Ui, spk: &mut tts::Tts) {
        egui::ComboBox::from_label("Numbers or sentences?")
            .selected_text(self.config.exercise_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.config.exercise_type,
                    ExerciseType::Words,
                    "Sentences",
                );
                ui.selectable_value(
                    &mut self.config.exercise_type,
                    ExerciseType::Numbers,
                    "Numbers",
                );
            });

        if self.config.exercise_type == ExerciseType::Words {
            egui::ComboBox::from_label("Select sentences file")
                .selected_text(
                    self.file
                        .sel_file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                )
                .show_ui(ui, |ui| {
                    let files = self.file.dirfiles.clone();
                    for file in files {
                        let filename = file
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_owned();
                        if ui
                            .selectable_value(&mut self.file.sel_file_path, file, &filename)
                            .clicked()
                        {
                            self.get_file();
                        };
                    }
                });
        };

        // normal stuff
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button("Pick").clicked() {
                    self.pick_next();
                }
                if ui.button("Speak").clicked() {
                    self.say(spk);
                }
                if ui.button("Clear").clicked() {
                    self.sequence.clear();
                }
                ui.checkbox(&mut self.seq_show, "Show sentence");
            });
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Sequence length");
                    ui.add(egui::Slider::new(&mut self.seq_length, 1..=10));
                    ui.end_row();
                });
            if self.seq_show {
                ui.separator();
                egui::Grid::new("answer_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Sentence");
                        ui.heading(&self.sequence.to_string());
                        ui.end_row();

                        ui.label("Reversed");
                        ui.label(&self.sequence_rev);
                        ui.end_row();

                        ui.label("Alphabetical");
                        ui.label(&self.sequence_alpha);
                        ui.end_row();

                        ui.label("Alphabetical reversed");
                        ui.label(&self.sequence_alpha_rev);
                        ui.end_row();
                    });
            }
        });
    }
}
