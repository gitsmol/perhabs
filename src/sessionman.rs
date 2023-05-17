use crate::{
    asset_loader::AppData,
    menu,
    sessions::{
        cog_numbers::CogNumbers, cog_words::CogWords, seq_numbers::NumSeq,
        spatial_drawing::SpatialDrawing, vergence::Vergence,
    },
};

use tts::Tts;

/// Stores all available exercises (sessions). In order to only display one, store its name in open
/// and only display the session matching that str.
pub struct SessionManager {
    pub sessions: Vec<Box<dyn Exercise>>,
    pub open: Option<&'static str>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            sessions: vec![
                Box::new(CogNumbers::default()),
                Box::new(CogWords::default()),
                Box::new(Vergence::default()),
                Box::new(NumSeq::default()),
                Box::new(SpatialDrawing::default()),
            ],
            open: None,
        }
    }
}

impl SessionManager {
    pub fn buttons(&mut self, ui: &mut egui::Ui) {
        for session in &self.sessions {
            if menu::menu_button(ui, session.name(), session.description()).clicked() {
                self.open = Some(session.name());
            };
        }
    }

    pub fn buttons_cols(&mut self, ui: &mut egui::Ui) {
        let buttons: f32 = self.sessions.len() as f32;
        let col_1_range = buttons - (buttons / 2.).floor();

        ui.columns(2, |col| {
            for i in 0..col_1_range as usize {
                if let Some(session) = self.sessions.get(i) {
                    if menu::menu_button(&mut col[0], session.name(), session.description())
                        .clicked()
                    {
                        self.open = Some(session.name());
                    };
                };
            }

            for i in col_1_range as usize..buttons as usize {
                if let Some(session) = self.sessions.get(i) {
                    if menu::menu_button(&mut col[1], session.name(), session.description())
                        .clicked()
                    {
                        self.open = Some(session.name());
                    };
                };
            }
        });
    }

    pub fn session_show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        // What is the currently open session?
        let name = match self.open {
            Some(name) => name,
            None => return,
        };
        for session in &mut self.sessions {
            // Only show the currently open session.
            if name == session.name() {
                session.show(ctx, appdata, tts)
            }
        }
    }
}

/// Something to view
pub trait Exercise {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts);
}
