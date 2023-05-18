#![warn(clippy::all)]
use crate::{
    modules::asset_loader::AppData,
    modules::{clock, debug_info, rand_timer},
};
use egui::{Context, Ui};
use std::collections::BTreeSet;
use tts::Tts;

pub struct Windows {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub windows: Vec<Box<dyn AppWin>>,
    pub open: BTreeSet<String>,
}

impl Default for Windows {
    fn default() -> Self {
        Self::from_windows(vec![
            // Box::new(multitasker::MultiTasker::default()),
            #[cfg(not(target_arch = "wasm32"))]
            Box::new(rand_timer::RandTimer::default()), // WASM doesn't support threading
            Box::new(clock::Clock::default()),
            Box::new(debug_info::DebugInfo::default()),
        ])
    }
}

impl Windows {
    pub fn from_windows(windows: Vec<Box<dyn AppWin>>) -> Self {
        let open = BTreeSet::new();
        // open.insert(rand_timer::RandTimer::default().name().to_owned());
        // open.insert(sequences::Sequences::default().name().to_owned());
        // open.insert(debug_info::DebugInfo::default().name().to_owned());
        Self { windows, open }
    }

    pub fn labels(&mut self, ui: &mut Ui) {
        let Self { windows, open } = self;
        for window in windows {
            let mut is_open = open.contains(window.name());
            ui.toggle_value(&mut is_open, window.name());
            set_open(open, window.name(), is_open);
        }
    }

    pub fn windows(&mut self, ctx: &Context, appdata: &AppData, tts: &mut Tts) {
        let Self { windows, open } = self;
        for window in windows {
            let mut is_open = open.contains(window.name());
            if is_open == true {
                window.show(ctx, &mut is_open, appdata, tts);
                set_open(open, window.name(), is_open);
            }
        }
    }
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut Tts);
}

/// Something to view
pub trait AppWin {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, appdata: &AppData, tts: &mut Tts);
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
