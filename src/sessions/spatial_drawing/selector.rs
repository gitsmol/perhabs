use std::path::Path;

use egui::{emath, Color32, Frame, Pos2, Rect, Response, Sense};
use perhabs::write_string_to_file;

use crate::{
    asset_loader::AppData,
    sessions::spatial_drawing::{painters::PuzzleGrid, SessionStatus},
};

use super::painters::Puzzle;

// All the selector functions go here.
impl super::SpatialDrawing {
    /// Controls used only in selector window
    fn ui_selector_controls(&mut self, ui: &mut egui::Ui, appdata: &AppData) {
        ui.label(&self.status_text);
        if ui.button("Export to file").clicked() {
            let now = chrono::Local::now().to_string();
            let filename = format!("exercise_configs_{}.json", now);
            self.export_puzzles_to_json(filename.as_str(), appdata);
        }
    }

    /// Shows an editable exercise
    fn ui_mini(&self, ui: &mut egui::Ui, exercise: &Puzzle) -> Response {
        // Setup
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());

        if response.clicked() {
            response.mark_changed();
        }

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );

        painter.extend(exercise.shapes(&to_screen, 6., Color32::KHAKI));
        painter.extend(self.puzzle_grid.shapes(&to_screen, 3., Color32::WHITE));

        response
    }

    /// Pick a puzzle and start exercise.
    fn pick_puzzle(&mut self, puzzle: &Puzzle) {
        debug!("Starting spatial puzzle.");
        self.puzzle = puzzle.to_owned();
        self.state = SessionStatus::Exercising;
        self.puzzle_grid = PuzzleGrid::new(puzzle.size())
    }

    /// Pick a puzzle and edit it.
    fn edit_puzzle(&mut self, puzzle: &Puzzle) {
        debug!("Editing spatial puzzle.");
        self.puzzle = puzzle.to_owned();
        self.state = SessionStatus::Editing;
    }

    pub fn export_puzzles_to_json(&mut self, path: &str, appdata: &AppData) {
        // Create a vec containing all puzzles, both from the existing excconfig
        // and puzzels that were created in the editor and stored in memory (in puzzle_list)
        if let Some(excconfig) = &appdata.excconfig {
            let mut new_excconfig = excconfig.clone();
            new_excconfig
                .spatial_drawing
                .extend(self.puzzle_edit_list.to_owned());

            if let Ok(serialized) = serde_json::to_string(&new_excconfig) {
                match write_string_to_file(&Path::new(path), serialized) {
                    Ok(_) => self.status_text = String::from("Puzzles succesfully exported."),
                    Err(e) => {
                        self.status_text = String::from(format!("Error exporting puzzles: {:?}", e))
                    }
                };
            };
        }
    }

    fn puzzle_list(&mut self, ui: &mut egui::Ui, list: &Vec<Puzzle>) {
        let mut i = 0;

        for puzzle in list {
            ui.centered_and_justified(|ui| {
                Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    if self
                        .ui_mini(ui, puzzle)
                        // Show right-click menu
                        .context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                self.edit_puzzle(puzzle);
                            }
                        })
                        // On click, start puzzle
                        .clicked()
                    {
                        self.pick_puzzle(puzzle);
                    }
                });
            });

            // Start a new row if things get too wide.
            i += 1;
            if i > 5 {
                ui.end_row();
            }
        }
    }

    /// Select a puzzle to play or edit.
    pub fn ui_selector(&mut self, ui: &mut egui::Ui, _: &mut tts::Tts, appdata: &AppData) {
        // Add basic controls
        self.ui_selector_controls(ui, appdata);

        egui::ScrollArea::new([true, true]).show(ui, |ui| {
            let box_size = ui.available_width() * 0.2;
            egui::Grid::new("puzzle_selector")
                .min_row_height(box_size)
                .min_col_width(box_size)
                .max_col_width(box_size)
                .show(ui, |ui| {
                    if let Some(config) = &appdata.excconfig {
                        self.puzzle_list(ui, &config.spatial_drawing);
                    }

                    ui.end_row();
                    if self.puzzle_edit_list.len() > 0 {
                        self.puzzle_list(ui, &self.puzzle_edit_list.to_owned());
                    }

                    for i in 5..=7 {
                        ui.vertical_centered_justified(|ui| {
                            Frame::dark_canvas(ui.style()).show(ui, |ui| {
                                if self.ui_mini(ui, &Puzzle::new(i)).clicked() {
                                    {
                                        debug!("Starting spatial exercise.");
                                        self.puzzle = Puzzle::new(i);
                                        self.state = SessionStatus::Editing;
                                    };
                                }
                            });

                            if ui.button("New").clicked() {
                                debug!("Adding new drawing.");
                                self.puzzle = Puzzle::new(i);
                                self.state = SessionStatus::Editing;
                            }
                        });
                    }
                });
        });
    }
}
