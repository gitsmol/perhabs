use std::collections::HashMap;

use egui::{
    emath::RectTransform,
    epaint::{CircleShape, RectShape},
    pos2, vec2, Color32, Pos2, Rect, Rounding, Shape,
};

/// A struct containing a hashmap of the positions of circles guiding a drawing exercise.
/// The coordinates to the circles are stored in two nested vectors in X, Y order.
/// The hashmap starts out empty. The positions for a given key (grid size) are calculated
/// on demand and thereafter read from the hashmap.
///
/// The positions calculated are the intersections ('crossings') on a grid. For most
/// purposes, they can practically be used as the centers of a square *between* gridlines.
pub struct Grid {
    positions: HashMap<usize, Vec<Vec<Pos2>>>,
}

impl Grid {
    /// Create guide circles on an evenly spaced grid of given size.
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    /// Generate the positions of the crossings on the grid as normalized coordinates
    /// So the top-left crossing would be at (0.0, 0.0). The next one at (0.1, 0.1) etc...
    /// To draw to screen, use a [`RectTransform`] to find the correct scaling of the
    /// normalized values.
    fn gen_coords(&self, grid_size: usize) -> Vec<Vec<Pos2>> {
        let mut rows_pos = vec![];
        let mut cols_pos = vec![];

        let rows = grid_size;
        let columns = grid_size;

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

    /// Tries to get all crossing coordinates for the grid
    pub fn get_all_coords(&mut self, grid_size: usize) -> Vec<Vec<Pos2>> {
        // Can we find the positions for the given grid size?
        match self.positions.get(&grid_size) {
            // Yes, return positions
            Some(positions) => positions.to_owned(),
            // No, calculate positions first, then retry.
            None => {
                debug!("Calculating `Grid` positions for size {}", grid_size);
                self.positions.insert(grid_size, self.gen_coords(grid_size));
                self.get_all_coords(grid_size)
            }
        }
    }

    /// Retrieve the (normalized) coordinates of a crossing on the grid.
    pub fn get_coordinate(&mut self, grid_size: usize, column: usize, row: usize) -> Option<Pos2> {
        let positions = self.get_all_coords(grid_size);
        if let Some(row) = positions.get(row) {
            if let Some(coord) = row.get(column) {
                return Some(coord.to_owned());
            }
        }
        None
    }

    /// Check if a given coordinate matches a crossing and return a ref to that coord.
    /// The given tolerance determines how far from the center of the crossing the match
    /// will still be considered valid.
    pub fn match_coords(&self, grid_size: usize, coord: Pos2, tolerance: f32) -> Option<&Pos2> {
        if let Some(positions) = self.positions.get(&grid_size) {
            for row in positions {
                for pos in row {
                    let in_bound_x = (coord.x - tolerance..=coord.x + tolerance).contains(&pos.x);
                    let in_bound_y = (coord.y - tolerance..=coord.y + tolerance).contains(&pos.y);
                    if in_bound_x && in_bound_y {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// Recursive function that calculates shapes for the given grid size. If the given
    /// grid size is not already stored in the positions hashmap, add the positions and
    /// recurse into this function.
    ///
    /// The default shape is a circle. Setting squircle to true generates squircle shapes.
    /// # Arguments
    ///
    /// * `grid_size` - The size of the grid for which to draw shapes.
    /// * `screen` - A transformation that maps normalized coordinates to screen coordinates.
    /// * `rel_size` - The relative size of the squircles as a fraction of 1 (1 = 100%)
    /// * `color` - The color of the squircles.
    /// * `squircle` - Make the shape a squircle. Defaults to false.
    pub fn draw_shapes(
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
                debug!("Calculating grid positions for size {}", grid_size);
                self.positions.insert(grid_size, self.gen_coords(grid_size));
                self.draw_shapes(grid_size, screen, rel_size, color, squircle)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_coords_2x2() {
        let grid = Grid::new();
        let coords = grid.gen_coords(2);
        assert_eq!(coords.len(), 2);
        assert_eq!(coords[0].len(), 2);
        assert_eq!(coords[0][0], pos2(0.333333333, 0.3333333333));
    }

    #[test]
    fn test_gen_coords_3x3() {
        // [0] = x
        // [1] = y
        let grid = Grid::new();
        let coords = grid.gen_coords(3);
        assert_eq!(coords.len(), 3);
        assert_eq!(coords[0].len(), 3);
        assert_eq!(coords[1].len(), 3);
        assert_eq!(coords[0][0], pos2(0.25, 0.25));
        assert_eq!(coords[0][1], pos2(0.25, 0.5));
        assert_eq!(coords[1][0], pos2(0.5, 0.25));
        assert_eq!(coords[1][1], pos2(0.5, 0.5));
    }
}
