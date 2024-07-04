use crate::{exercises::*, shared::AppData, widgets::menu_button};

use egui::vec2;
use tts::Tts;

/// Stores all available exercises (sessions). In order to only display one, store its name in open
/// and only display the session matching that static str.
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
                Box::new(NumSeq::default()),
                Box::new(EpisodicMemory::default()),
                Box::new(SpatialDrawing::default()),
                Box::new(Vergence::default()),
                Box::new(DepthPerception::default()),
                Box::new(BinoSaccades::default()),
                Box::new(VisualAlignment::default()),
                Box::new(VisRecognition::default()),
                Box::new(VisSaccades::default()),
                Box::new(NumberedSquares::default()),
                Box::new(ContainerSearch::default()), // Box::new(SpatialHearing::default()),
            ],
            open: None,
        }
    }
}

impl SessionManager {
    pub fn buttons(&mut self, ui: &mut egui::Ui) {
        for session in &self.sessions {
            if menu_button(ui, None, None, session.name(), session.description()).clicked() {
                self.open = Some(session.name());
            };
        }
    }

    pub fn buttons_cols(&mut self, ui: &mut egui::Ui) {
        let buttons_total: f32 = self.sessions.len() as f32;
        let col_1_range = buttons_total - (buttons_total / 2.).floor();

        ui.columns(2, |col| {
            // Column 1 gets populated with at least half the buttons
            for i in 0..col_1_range as usize {
                if let Some(session) = self.sessions.get(i) {
                    let desired_width = col[0].available_width();
                    if menu_button(
                        &mut col[0],
                        Some(vec2(desired_width, 90.)),
                        None,
                        session.name(),
                        session.description(),
                    )
                    .clicked()
                    {
                        self.open = Some(session.name());
                    };
                };
            }

            // Column 2 gets populated with the remaining buttons
            for i in col_1_range as usize..buttons_total as usize {
                if let Some(session) = self.sessions.get(i) {
                    let desired_width = col[1].available_width();

                    if menu_button(
                        &mut col[1],
                        Some(vec2(desired_width, 90.)),
                        None,
                        session.name(),
                        session.description(),
                    )
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

/// An Exercise has a menu and a session window. It opens/closes by its name. A description is nice.
pub trait Exercise {
    // `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn help(&self) -> &'static str;

    /// Entry into the ui codepath.
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts);

    /// Shows the exercise menu.
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut Tts);

    /// Shows the exercise session i.e. the exercise as it is being trained.
    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut Tts);

    // To make sure we can clean up when quitting session.
    fn reset(&mut self);
}
