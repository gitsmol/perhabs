use crate::{
    asset_loader::AppData,
    sessions::{
        cog_numbers::CogNumbers, cog_words::CogWords, seq_numbers::NumSeq, vergence::Vergence,
    },
};
use egui::{Context, Ui};
use tts::Tts;

pub struct Sessions {
    pub sessions: Vec<Box<dyn SessionPanel>>,
    pub open: Option<&'static str>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            sessions: vec![
                Box::new(CogNumbers::default()),
                Box::new(CogWords::default()),
                Box::new(Vergence::default()),
                Box::new(NumSeq::default()),
            ],
            open: None,
        }
    }
}

impl Sessions {
    pub fn buttons(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            for session in &self.sessions {
                let name = session.name();
                if ui.button(session.name()).clicked() {
                    self.open = Some(name)
                }
            }
        });
    }

    pub fn sessions(&mut self, ctx: &Context, appdata: &AppData, tts: &mut Tts) {
        let name = match self.open {
            Some(name) => name,
            None => return,
        };
        for session in &mut self.sessions {
            if name == session.name() {
                session.show(ctx, appdata, tts)
            }
        }
    }
}

/// Something to view
pub trait SessionPanel {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts);
}
