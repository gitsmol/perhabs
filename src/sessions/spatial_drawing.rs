use std::path::Path;

use crate::{asset_loader::AppData, sessionman::Exercise};
use egui::{CentralPanel, Color32, Frame, Pos2, Stroke};
mod editor;
mod exercise;
pub mod painters;
mod selector;
use log::debug;
use perhabs::write_string_to_file;

use self::painters::{Puzzle, PuzzleGrid};

#[derive(PartialEq, Eq)]
enum Transformation {
    HMirror,
    VMirror,
    LeftTilt,
    RightTilt,
}

impl std::fmt::Display for Transformation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Transformation::HMirror => write!(f, "Mirror horizontally"),
            Transformation::VMirror => write!(f, "Mirror vertically"),
            Transformation::LeftTilt => write!(f, "Tilt left"),
            Transformation::RightTilt => write!(f, "Tilt right"),
        }
    }
}

#[derive(PartialEq, Eq)]
enum SessionStatus {
    Selecting,
    Editing,
    Exercising,
    Reviewing,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpatialDrawing {
    state: SessionStatus,
    puzzle_grid: PuzzleGrid,
    puzzle_list: Vec<Puzzle>,
    puzzle: Puzzle,
    puzzle_transform: Transformation,
    user_drawing: Vec<Vec<Pos2>>,
    stroke: Stroke,
    status_text: String,
}

impl Default for SpatialDrawing {
    fn default() -> Self {
        Self {
            state: SessionStatus::Selecting,
            puzzle_grid: PuzzleGrid::new(5),
            puzzle_list: vec![],
            puzzle: Puzzle::new(5),
            puzzle_transform: Transformation::RightTilt,
            user_drawing: Default::default(),
            stroke: Stroke::new(5.0, Color32::from_rgb(25, 200, 100)),
            status_text: String::from("Pick an exercise."),
        }
    }
}

impl SpatialDrawing {
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.state = SessionStatus::Selecting
            };
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            egui::ComboBox::from_label("Select transformation")
                .selected_text(format!("{}", self.puzzle_transform))
                .show_ui(ui, |ui| {
                    for item in [
                        Transformation::HMirror,
                        Transformation::VMirror,
                        Transformation::LeftTilt,
                        Transformation::RightTilt,
                    ] {
                        let item_label = item.to_string();
                        ui.selectable_value(&mut self.puzzle_transform, item, item_label);
                    }
                });
            if ui.button("Clear Painting").clicked() {
                self.user_drawing.clear();
            }
        });
        // Force the columns layout to be square.
        if ui.available_width() > ui.available_height() * 2. {
            ui.add_space(ui.available_height() * 0.05);
        } else {
            ui.add_space(ui.available_height() - ui.available_width() / 2.);
        }
    }

    fn ui_editor_controls(&mut self, ui: &mut egui::Ui, _: &AppData) {
        ui.horizontal(|ui| {
            if ui.button("Clear editor").clicked() {
                self.puzzle.clear();
            }

            if ui.button("Save").clicked() {
                self.puzzle_list.push(self.puzzle.to_owned());
            }
        });
    }

    fn ui_selector_controls(&mut self, ui: &mut egui::Ui, appdata: &AppData) {
        ui.label(&self.status_text);
        if ui.button("Export to file").clicked() {
            // Create a vec containing all puzzles, both from the existing excconfig
            // and puzzels that were created in the editor and stored in memory (in puzzle_list)
            let mut all_puzzles = self.puzzle_list.clone();
            if let Some(excconfig) = &appdata.excconfig {
                all_puzzles.extend(excconfig.spatial_drawing.to_owned());
            }

            if let Ok(serialized) = serde_json::to_string(&all_puzzles) {
                match write_string_to_file(&Path::new("temp/puzzles.json"), serialized) {
                    Ok(_) => self.status_text = String::from("Puzzles succesfully exported."),
                    Err(e) => {
                        self.status_text = String::from(format!("Error exporting puzzles: {:?}", e))
                    }
                };
            };
        }
    }

    /// Select a puzzle to play or edit.
    fn ui_selector(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts, appdata: &AppData) {
        self.ui_selector_controls(ui, appdata);
        egui::ScrollArea::new([true, true]).show(ui, |ui| {
            let box_size = ui.available_width() * 0.2;
            egui::Grid::new("puzzle_selector")
                .min_row_height(box_size)
                .min_col_width(box_size)
                .max_col_width(box_size)
                .show(ui, |ui| {
                    let mut i = 0;
                    if let Some(config) = &appdata.excconfig {
                        for puzzle in &config.spatial_drawing {
                            ui.vertical_centered_justified(|ui| {
                                Frame::dark_canvas(ui.style()).show(ui, |ui| {
                                    if self.ui_mini(ui, puzzle).clicked() {
                                        {
                                            debug!("Starting spatial puzzle.");
                                            self.puzzle = puzzle.to_owned();
                                            self.state = SessionStatus::Exercising;
                                        };
                                    }
                                });

                                if ui.button("Edit").clicked() {
                                    debug!("Editing puzzle.");
                                    self.puzzle = puzzle.to_owned();
                                    self.state = SessionStatus::Editing;
                                }
                            });

                            // Start a new row if things get too wide.
                            i += 1;
                            if i > 3 {
                                ui.end_row();
                            }
                        }
                    }

                    ui.end_row();
                    for puzzle in &self.puzzle_list {
                        if Frame::dark_canvas(ui.style())
                            .show(ui, |ui| self.ui_mini(ui, puzzle))
                            .response
                            .clicked()
                        {
                            self.puzzle = puzzle.to_owned();
                            self.state == SessionStatus::Exercising;
                        };
                        if ui.button("Edit").clicked() {
                            debug!("Editing puzzle.");
                            self.puzzle = puzzle.to_owned();
                            self.state = SessionStatus::Editing;
                        }
                        i += 1;
                        if i % 3 == 0 {
                            ui.end_row();
                        }
                    }
                    ui.end_row();

                    ui.vertical_centered_justified(|ui| {
                        Frame::dark_canvas(ui.style()).show(ui, |ui| {
                            if self.ui_mini(ui, &Puzzle::new(5)).clicked() {
                                {
                                    debug!("Starting spatial exercise.");
                                    self.puzzle = Puzzle::new(5);
                                    self.state = SessionStatus::Exercising;
                                };
                            }
                        });

                        if ui.button("New").clicked() {
                            debug!("Adding new drawing.");
                            self.puzzle = Puzzle::new(5);
                            self.state = SessionStatus::Editing;
                        }
                    });
                });
        });
    }

    /// Edit or create a puzzle and store it.
    fn ui_editor(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts, appdata: &AppData) {
        // Show controls at the top
        self.ui_controls(ui);
        self.ui_editor_controls(ui, appdata);

        // Left column shows example, right column is where user draws and reviews.
        ui.columns(2, |cols| {
            Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| self.ui_editable(ui));
            Frame::dark_canvas(cols[1].style()).show(&mut cols[1], |ui| self.ui_drawing(ui));
        });
    }

    /// Show the session (fullscreen)
    fn ui_exercise(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts, _: &AppData) {
        // Show controls at the top
        self.ui_controls(ui);
        // Force the columns layout to be square.
        ui.add_space(ui.available_height() - ui.available_width() / 2.);

        // Left column shows example, right column is where user draws and reviews.
        ui.columns(2, |cols| {
            Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| self.ui_example(ui));
            Frame::dark_canvas(cols[1].style()).show(&mut cols[1], |ui| self.ui_drawing(ui));
        });
    }
}

impl Exercise for SpatialDrawing {
    fn name(&self) -> &'static str {
        "Spatial drawing"
    }

    fn description(&self) -> &'static str {
        "Draw a shape mirrored or on its side."
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut tts::Tts) {
        match self.state {
            SessionStatus::Selecting => {
                CentralPanel::default().show(ctx, |ui| self.ui_selector(ui, tts, appdata))
            }
            SessionStatus::Editing => {
                CentralPanel::default().show(ctx, |ui| self.ui_editor(ui, tts, appdata))
            }
            SessionStatus::Exercising => {
                CentralPanel::default().show(ctx, |ui| self.ui_exercise(ui, tts, appdata))
            }
            SessionStatus::Reviewing => {
                CentralPanel::default().show(ctx, |ui| self.ui_exercise(ui, tts, appdata))
            }
        };
    }
}
