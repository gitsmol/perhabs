use egui::{emath::RectTransform, epaint::CircleShape, pos2, Color32, Pos2, Shape, Stroke};
use log::debug;
use serde::{Deserialize, Serialize};

/// A puzzle is a collection of lines on a grid in normalized (0.0 - 1.0) coordinates.
/// The puzzle can be completely cleared. Lines can be added one by one by
/// inputting points (pos2) using edit.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Puzzle {
    grid_size: usize,
    lines: Vec<[Pos2; 2]>,
    #[serde(skip)]
    edit_line: Option<Pos2>,
}

impl Puzzle {
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

/// A struct containing the positions of circles guiding a drawing exercise.
/// The coordinates to the circles are stored in two nested vectors in X, Y order.
pub struct PuzzleGrid {
    pub positions: Vec<Vec<Pos2>>,
}

impl PuzzleGrid {
    /// Create guide circles on an evenly spaced grid of given size.
    pub fn new(grid_size: usize) -> Self {
        debug!("Initializing GuideCircles");
        let mut rows_pos = vec![];
        let mut cols_pos = vec![];

        // Add 1 to row and column to account for skipping the first and the last ones.
        // Counting from zero compensates for the first one we skip.
        // The first and last are skipped because they would be exactly on the edges
        // of the coordinate system, having x or y of 0 or 1.
        let rows = grid_size + 1;
        let columns = grid_size + 1;

        for i in 0..rows {
            let x: f32 = i as f32 / rows as f32;
            if x % 1.0 != 0.0 {
                for i in 0..columns {
                    let y: f32 = i as f32 / columns as f32;
                    if y % 1.0 != 0.0 {
                        let pos = pos2(x, y);
                        cols_pos.push(pos);
                    }
                }
                rows_pos.push(cols_pos.to_owned());
                cols_pos = vec![];
            }
        }
        Self {
            positions: rows_pos,
        }
    }

    /// Retrieve the (normalized) coordinates of a circle's center.
    pub fn get_pos(&self, column: usize, row: usize) -> Option<&Pos2> {
        if let Some(row) = self.positions.get(row) {
            if let Some(coord) = row.get(column) {
                return Some(coord);
            }
        }
        None
    }

    /// Check if a given coordinate matches a guide circle and
    /// return a reference to that circle's position.
    pub fn match_coords(&self, coord: Pos2) -> Option<&Pos2> {
        // Set 1% tolerance (ie how big is the clickable square)
        let tolerance = 0.01;
        for row in &self.positions {
            for pos in row {
                if coord.x - tolerance < pos.x && coord.x + tolerance > pos.x {
                    if coord.y - tolerance < pos.y && coord.y + tolerance > pos.y {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// Return shapes for the guide circles, to be used with egui::Painter.
    pub fn shapes(&self, screen: &RectTransform, size: f32, color: Color32) -> Vec<Shape> {
        let mut shapes = vec![];
        for row in &self.positions {
            for pos in row {
                let pos_on_screen = screen * pos.to_owned();
                let circle = egui::Shape::Circle(CircleShape::filled(pos_on_screen, size, color));
                shapes.push(circle);
            }
        }
        shapes
    }
}
