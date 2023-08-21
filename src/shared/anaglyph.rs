use egui::Color32;

pub enum Eye {
    Left,
    Right,
}

#[derive(PartialEq, Clone)]
pub struct AnaglyphColor {
    pub left: Color32,
    pub right: Color32,
    pub calibration_left: Color32,
    pub calibration_right: Color32,
}

impl Default for AnaglyphColor {
    fn default() -> Self {
        Self {
            left: Color32::from_rgba_unmultiplied(0, 38, 230, 100), // blue
            right: Color32::from_rgba_unmultiplied(255, 25, 25, 50), // red
            calibration_left: Color32::from_rgba_unmultiplied(0, 38, 230, 100), // blue,
            calibration_right: Color32::from_rgba_unmultiplied(255, 25, 25, 50), // red,
        }
    }
}

impl AnaglyphColor {
    /// Reset calibration values to currently in-use values
    pub fn reset_calibration_values(&mut self) {
        self.calibration_left = self.left.clone();
        self.calibration_right = self.right.clone();
    }

    /// Save calibrated values and stop calibration
    pub fn save_calibration_colors(&mut self) {
        self.left = self.calibration_left;
        self.right = self.calibration_right;
    }

    /// Swap right and left eye colors
    pub fn swap_calibration_colors(&mut self) {
        let tmp = self.calibration_left;
        self.calibration_left = self.calibration_right;
        self.calibration_right = tmp;
    }
}
