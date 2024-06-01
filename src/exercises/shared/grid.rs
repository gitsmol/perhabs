use std::collections::HashMap;

use egui::{
    emath::{self, RectTransform},
    epaint::{CircleShape, RectShape},
    pos2, vec2, Color32, Pos2, Rect, Response, Rounding, Sense, Shape,
};

/// A struct containing a hashmap of the positions of circles guiding a drawing exercise.
/// The coordinates to the circles are stored in two nested vectors in X, Y order.
/// The hashmap starts out empty. The positions for a given key (grid size) are calculated
/// on demand and thereafter read from the hashmap.
pub struct Grid {
    size: usize,
    positions: HashMap<usize, Vec<Vec<Pos2>>>,
}

impl Grid {
    /// Create guide circles on an evenly spaced grid of given size.
    pub fn new(size: usize) -> Self {
        Self {
            positions: HashMap::new(),
            size,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn gen_positions(&self, grid_size: usize) -> Vec<Vec<Pos2>> {
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
        rows_pos
    }

    fn get_positions(&mut self, grid_size: usize) -> Vec<Vec<Pos2>> {
        // Can we find the positions for the given grid size?
        match self.positions.get(&grid_size) {
            // Yes, calculate shapes.
            Some(positions) => positions.to_owned(),
            // No, calculate positions first, then retry.
            None => {
                debug!("Calculating PuzzleGrid positions for size {}", grid_size);
                self.positions
                    .insert(grid_size, self.gen_positions(grid_size));
                self.get_positions(grid_size)
            }
        }
    }

    /// Retrieve the (normalized) coordinates of a circle's center.
    pub fn get_coordinate(&mut self, grid_size: usize, column: usize, row: usize) -> Option<Pos2> {
        let positions = self.get_positions(grid_size);
        if let Some(row) = positions.get(row) {
            if let Some(coord) = row.get(column) {
                return Some(coord.to_owned());
            }
        }
        None
    }

    /// Check if a given coordinate matches a guide circle and
    /// return a reference to that circle's position.
    pub fn match_coords(&self, grid_size: usize, coord: Pos2) -> Option<&Pos2> {
        if let Some(positions) = self.positions.get(&grid_size) {
            // Set 1% tolerance (ie how big is the clickable square)
            let tolerance = 0.01;
            for row in positions {
                for pos in row {
                    if coord.x - tolerance < pos.x && coord.x + tolerance > pos.x {
                        if coord.y - tolerance < pos.y && coord.y + tolerance > pos.y {
                            return Some(pos);
                        }
                    }
                }
            }
        }
        None
    }

    /// Calculate shapes for the given grid size. If the given grid size is not already
    /// stored in the positions hashmap, add the positions and recurse into this function
    /// The default shape is a circle. Setting squircle to true generates squircle shapes.
    /// # Arguments
    ///
    /// * `grid_size` - The size of the grid for which to shapes.
    /// * `screen` - A transformation that maps normalized coordinates to screen coordinates.
    /// * `rel_size` - The relative size of the squircles as a fraction of 1 (1 = 100%)
    /// * `color` - The color of the squircles.
    /// * `squircle` - Make the shape a squircle. Defaults to false.
    pub fn shapes(
        &mut self,
        grid_size: usize,
        screen: &RectTransform,
        rel_size: f32,
        color: Color32,
        squircle: bool,
    ) -> Vec<Shape> {
        let abs_size: f32 = {
            let margin = 0.9;
            (rel_size / grid_size as f32) * margin * screen.scale().min_elem()
        };

        // Can we find the positions for the given grid size?
        match self.positions.get(&grid_size) {
            // Yes, calculate shapes.
            Some(positions) => {
                let mut shapes = vec![];
                for row in positions {
                    for pos in row {
                        let pos_on_screen = screen * pos.to_owned();
                        let shape: Shape = match squircle {
                            true => {
                                let rect =
                                    Rect::from_center_size(pos_on_screen, vec2(abs_size, abs_size));
                                egui::Shape::Rect(RectShape::filled(
                                    rect,
                                    Rounding::same(abs_size * 0.15),
                                    color,
                                ))
                            }
                            false => egui::Shape::Circle(CircleShape::filled(
                                pos_on_screen,
                                abs_size,
                                color,
                            )),
                        };
                        shapes.push(shape);
                    }
                }
                shapes
            }
            // No, calculate positions first, then retry.
            None => {
                debug!("Calculating PuzzleGrid positions for size {}", grid_size);
                self.positions
                    .insert(grid_size, self.gen_positions(grid_size));
                self.shapes(grid_size, screen, rel_size, color, squircle)
            }
        }
    }
}
