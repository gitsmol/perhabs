#![warn(clippy::all)]
use crate::modules::{clock, debug_info, rand_timer, seq_numbers, sequences, vergence};
use egui::{Context, Ui};
use std::collections::BTreeSet;

pub struct Windows {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub windows: Vec<Box<dyn AppWin>>,
    pub open: BTreeSet<String>,
}

impl Default for Windows {
    fn default() -> Self {
        Self::from_windows(vec![
            Box::new(seq_numbers::NumSeq::default()),
            Box::new(sequences::Sequences::default()),
            Box::new(vergence::Vergence::default()),
            Box::new(rand_timer::RandTimer::default()),
            Box::new(clock::Clock::default()),
            Box::new(debug_info::DebugInfo::default()),
        ])
    }
}

impl Windows {
    pub fn from_windows(windows: Vec<Box<dyn AppWin>>) -> Self {
        let mut open = BTreeSet::new();
        open.insert(vergence::Vergence::default().name().to_owned());
        // open.insert(debug_info::DebugInfo::default().name().to_owned());
        Self { windows, open }
    }

    pub fn labels(&mut self, ui: &mut Ui) {
        let Self { windows, open } = self;
        for window in windows {
            let is_open = open.contains(window.name());
            if ui.selectable_label(is_open, window.name()).clicked() {
                // set_open(open, window.name(), is_open);
            };
        }
    }

    pub fn checkboxes(&mut self, ui: &mut Ui) {
        let Self { windows, open } = self;
        for window in windows {
            let mut is_open = open.contains(window.name());
            ui.toggle_value(&mut is_open, window.name());
            set_open(open, window.name(), is_open);
        }
    }

    pub fn windows(&mut self, ctx: &Context, spk: &mut tts::Tts) {
        let Self { windows, open } = self;
        for window in windows {
            let mut is_open = open.contains(window.name());
            window.show(ctx, &mut is_open, spk);
            set_open(open, window.name(), is_open);
        }
    }
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, spk: &mut tts::Tts);
}

/// Something to view
pub trait AppWin {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, spk: &mut tts::Tts);
}

// Open and close windows
fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}
