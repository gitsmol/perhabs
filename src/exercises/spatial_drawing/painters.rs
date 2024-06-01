use std::collections::HashMap;

use egui::{emath::RectTransform, epaint::CircleShape, pos2, Color32, Pos2, Shape, Stroke};
use log::debug;
use serde::{Deserialize, Serialize};

/// A puzzle is a collection of lines on a grid in normalized (0.0 - 1.0) coordinates.
/// The puzzle can be completely cleared. Lines can be added one by one by
/// inputting points (pos2) using edit.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpatialPuzzle {
    grid_size: usize,
    lines: Vec<[Pos2; 2]>,
    #[serde(skip)]
    edit_line: Option<Pos2>,
}

impl SpatialPuzzle {
    /// Create a new puzzle. The grid size is not used for any calculations
    /// but needs to be stored with the puzzle so any visual representations
    /// match the grid size the puzzle was made for.
    pub fn new(grid_size: usize) -> Self {
        Self {
            grid_size,
            lines: vec![],
            edit_line: None,
        }
    }

    /// Start or finish a line by adding a point. If a point exists, the line is completed.
    /// If no point 1 exists, point 1 is added. A line is only added to
    /// the vec of lines when it is completed by giving a second point.
    pub fn edit(&mut self, new_pos: Pos2) {
        // Are we finishing a line?
        if let Some(pos1) = self.edit_line {
            // Make sure we are adding a different pos (don't act on double clicks/mistakes)
            if pos1 != new_pos {
                self.add(pos1, new_pos);
                self.edit_line = None;
            }
        }
        // If not, we must be starting a line.
        else {
            self.edit_line = Some(new_pos);
        }
    }

    /// Add a line to the vec of lines.
    pub fn add(&mut self, start: Pos2, end: Pos2) {
        self.lines.push([start, end]);
    }

    /// Add multiple lines to the vec of lines.
    pub fn extend(&mut self, coords: Vec<[Pos2; 2]>) {
        self.lines.extend(coords);
    }

    /// Remove all lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Remove last line.
    pub fn undo(&mut self) {
        self.lines.pop();
    }

    pub fn size(&self) -> usize {
        self.grid_size
    }

    /// Create shapes for all lines according to relative screensize, width and color.
    pub fn shapes(&self, screen: &RectTransform, width: f32, color: Color32) -> Vec<Shape> {
        self.draw_shapes(self.lines.clone(), screen, width, color)
    }

    /// Create shapes but mirror them horizontally.
    pub fn shapes_hmirror(&self, screen: &RectTransform, width: f32, color: Color32) -> Vec<Shape> {
        let lines: Vec<[Pos2; 2]> = self
            .lines
            .clone()
            .into_iter()
            .map(|line| -> [Pos2; 2] {
                let mut newline = line;
                newline[0].x = 1.0 - line[0].x;
                newline[1].x = 1.0 - line[1].x;
                newline
            })
            .collect();

        self.draw_shapes(lines, screen, width, color)
    }

    /// Create shapes but mirror them vertically.
    pub fn shapes_vmirror(&self, screen: &RectTransform, width: f32, color: Color32) -> Vec<Shape> {
        let lines: Vec<[Pos2; 2]> = self
            .lines
            .clone()
            .into_iter()
            .map(|line| -> [Pos2; 2] {
                let mut newline = line;
                newline[0].y = 1.0 - line[0].y;
                newline[1].y = 1.0 - line[1].y;
                newline
            })
            .collect();

        self.draw_shapes(lines, screen, width, color)
    }

    /// Create shapes but tilt them to the left.
    pub fn shapes_tilt_left(
        &self,
        screen: &RectTransform,
        width: f32,
        color: Color32,
    ) -> Vec<Shape> {
        let lines: Vec<[Pos2; 2]> = self
            .lines
            .clone()
            .into_iter()
            .map(|line| -> [Pos2; 2] {
                let mut newline = line;
                newline[0].x = newline[0].y;
                newline[0].y = 1.0 - line[0].x;
                newline[1].x = newline[1].y;
                newline[1].y = 1.0 - line[1].x;
                newline
            })
            .collect();

        self.draw_shapes(lines, screen, width, color)
    }

    /// Create shapes but tilt them to the right.
    pub fn shapes_tilt_right(
        &self,
        screen: &RectTransform,
        width: f32,
        color: Color32,
    ) -> Vec<Shape> {
        let lines: Vec<[Pos2; 2]> = self
            .lines
            .clone()
            .into_iter()
            .map(|line| -> [Pos2; 2] {
                let mut newline = line;
                newline[0].y = newline[0].x;
                newline[0].x = 1.0 - line[0].y;
                newline[1].y = newline[1].x;
                newline[1].x = 1.0 - line[1].y;
                newline
            })
            .collect();

        self.draw_shapes(lines, screen, width, color)
    }

    /// Takes a Vec containing arrays with two points. Returns a vec of shapes.
    fn draw_shapes(
        &self,
        lines: Vec<[Pos2; 2]>,
        screen: &RectTransform,
        width: f32,
        color: Color32,
    ) -> Vec<Shape> {
        lines
            .into_iter()
            .map(|mut l| {
                for coord in l.iter_mut() {
                    *coord = screen * *coord;
                }
                let shape = egui::Shape::LineSegment {
                    points: l,
                    stroke: Stroke { width, color },
                };
                shape
            })
            .collect()
    }
}
