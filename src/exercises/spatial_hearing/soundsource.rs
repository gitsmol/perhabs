use std::fmt::Display;

use egui::Rect;

use crate::shared::Pos3;

/// A soundsource mapped in both 2D and 3D space.
/// Coordinates are fixed in an array as x, y, z.
/// Pos3 is the normalized position of the soundsource in 3d space.
/// Rect is an optional value that stores the shape of the soundsource
/// on screen.
#[derive(PartialEq, Clone)]
pub struct SoundSource {
    pub coords: [usize; 3],
    pub pos3: Pos3,
    pub rect: Option<Rect>,
}

impl Display for SoundSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let coords: String = self.coords.iter().map(|u| u.to_string()).collect();
        write!(f, "Coords: {coords}")
    }
}

/// A function to match coordinates to hardcoded(!) pins on the ESP32.
pub fn match_coords_to_pin(coords: [usize; 3]) -> Option<usize> {
    match coords {
        [0, 0, 0] => Some(12),
        [0, 1, 0] => Some(14),
        _ => None,
    }
}
