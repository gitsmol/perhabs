use std::ops::{Add, Sub};

use egui::Pos2;
use rand::Rng;

/// Holds a 3D position. Implemented functions all normalize input to range 0.0 - 1.0.
#[derive(Clone, Copy, PartialEq)]
pub struct Pos3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Pos3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Generate a random point in 3D space, given the maximum size of each dimension.
    /// Note: all input is normalized to 1.0.
    pub fn random(x_size: usize, y_size: usize, z_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut normalize = |size: usize| -> f32 {
            let random_number_in_range = rng.gen_range(0..size);
            random_number_in_range as f32 / size as f32
        };
        let x = normalize(x_size);
        let y = normalize(y_size);
        let z = normalize(z_size);

        Self { x, y, z }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> f32 {
        self.z
    }
}

impl Sub<Pos2> for Pos3 {
    type Output = Pos3;

    /// Subtract a pos2 from a pos3 by subtracting x and y.
    fn sub(self, rhs: Pos2) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z,
        }
    }
}

impl Add<Pos2> for Pos3 {
    type Output = Pos3;

    /// Add a pos2 to a pos3 by adding x and y.
    fn add(self, rhs: Pos2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z,
        }
    }
}
