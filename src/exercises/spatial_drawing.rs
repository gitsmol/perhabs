use egui::{CentralPanel, Color32, Pos2, Stroke};
mod editor;
mod exercise;
pub mod painters;
mod selector;

use crate::{shared::AppData, wm::Exercise};

use self::painters::{PuzzleGrid, SpatialPuzzle};

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

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SessionStatus::Selecting => write!(f, "Selecting"),
            SessionStatus::Editing => write!(f, "Editing"),
            SessionStatus::Exercising => write!(f, "Exercising"),
            SessionStatus::Reviewing => write!(f, "Reviewing"),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpatialDrawing {
    state: SessionStatus,
    puzzle_grid: PuzzleGrid,
    puzzle_edit_list: Vec<SpatialPuzzle>,
    puzzle: SpatialPuzzle,
    puzzle_transform: Transformation,
    user_drawing: Vec<Vec<Pos2>>,
    stroke: Stroke,
    status_text: String,
}

impl Default for SpatialDrawing {
    fn default() -> Self {
        Self {
            state: SessionStatus::Selecting,
            puzzle_grid: PuzzleGrid::new(),
            puzzle_edit_list: vec![],
            puzzle: SpatialPuzzle::new(5),
            puzzle_transform: Transformation::RightTilt,
            user_drawing: Default::default(),
            stroke: Stroke::new(5.0, Color32::from_rgb(25, 200, 100)),
            status_text: String::from("Pick an exercise."),
        }
    }
}

impl SpatialDrawing {
    /// The controls shown when editing or doing a puzzle.
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.state = SessionStatus::Selecting;
                self.user_drawing.clear();
            };
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
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
            if ui.button("Undo").clicked() {
                if self.user_drawing.len() > 1 {
                    self.user_drawing.swap_remove(self.user_drawing.len() - 2);
                };
            }
            if ui.button("Review").clicked() {
                self.state = SessionStatus::Reviewing;
            }
            ui.label(format!("{}", self.state));
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

            if ui.button("Undo").clicked() {
                self.puzzle.undo();
            }

            if ui.button("Save").clicked() {
                self.puzzle_edit_list.push(self.puzzle.to_owned());
            }
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

    fn reset(&mut self) {
        *self = SpatialDrawing::default();
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

    fn ui(&mut self, _: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {}

    fn session(&mut self, _: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {}
}
