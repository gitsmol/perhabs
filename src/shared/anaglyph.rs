use egui::Color32;

pub enum Eye {
    Left,
    Right,
}

#[derive(PartialEq, Clone)]
pub struct AnaglyphColor {
    pub left: Color32,
    pub right: Color32,
}

impl Default for AnaglyphColor {
    fn default() -> Self {
        Self {
            left: Color32::from_rgba_unmultiplied(0, 38, 230, 100), // blue
            right: Color32::from_rgba_unmultiplied(255, 25, 25, 50), // red
        }
    }
}
