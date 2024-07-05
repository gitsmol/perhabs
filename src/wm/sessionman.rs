use std::rc::Rc;

use crate::{
    exercises::*,
    shared::AppData,
    widgets::{self, menu_button},
};

use egui::vec2;
use tts::Tts;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExerciseType {
    Cognitive,
    Visual,
    Auditory,
}

impl std::fmt::Display for ExerciseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExerciseType::Cognitive => write!(f, "Cognitive"),
            ExerciseType::Visual => write!(f, "Visual"),
            ExerciseType::Auditory => write!(f, "Auditory"),
        }
    }
}

/// Stores all available exercises (sessions). In order to only display one, store its name in open
/// and only display the session matching that static str.
pub struct SessionManager {
    pub sessions: Vec<Box<dyn Exercise>>,
    pub selected_types: Vec<ExerciseType>,
    pub selected_sessions: Vec<usize>,
    pub open_session: Option<&'static str>,
}

impl Default for SessionManager {
    fn default() -> Self {
        let sessions: Vec<Box<dyn Exercise>> = vec![
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
            Box::new(ContainerSearch::default()),
            // Box::new(SpatialHearing::default()),
        ];
        Self {
            selected_sessions: vec![],
            sessions,
            selected_types: Vec::new(),
            open_session: None,
        }
    }
}

impl SessionManager {
    fn exercise_type_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for exercise_type in [
                ExerciseType::Cognitive,
                ExerciseType::Visual,
                ExerciseType::Auditory,
            ] {
                let is_selected = self.selected_types.contains(&exercise_type);
                let override_color = match is_selected {
                    true => Some(ui.visuals().selection.bg_fill),
                    false => None,
                };
                if widgets::menu_button(
                    ui,
                    Some(vec2(100., 50.)),
                    override_color,
                    exercise_type.to_string().as_str(),
                    "".into(),
                )
                .clicked()
                {
                    if is_selected {
                        self.selected_types.retain(|&t| t != exercise_type);
                    } else {
                        self.selected_types.push(exercise_type);
                    }
                    // Update selected exercises based on selected types
                    self.selected_sessions = self
                        .sessions
                        .iter()
                        .enumerate()
                        .filter(|(_, session)| {
                            session
                                .excercise_type()
                                .iter()
                                .any(|t| self.selected_types.contains(t))
                        })
                        .map(|(index, _)| index)
                        .collect();
                }
            }
        });
    }

    pub fn exercise_buttons(&mut self, ui: &mut egui::Ui) {
        for session in &self.sessions {
            if menu_button(ui, None, None, session.name(), session.description()).clicked() {
                self.open_session = Some(session.name());
            };
        }
    }

    pub fn exercise_buttons_cols(&mut self, ui: &mut egui::Ui) {
        // Display the exercise type selector first at the top.
        self.exercise_type_selector(ui);

        let buttons_total: f32 = self.selected_sessions.len() as f32;
        let col_1_range = buttons_total - (buttons_total / 2.).floor();

        ui.columns(2, |col| {
            // Column 1 gets populated with at least half the buttons
            for i in 0..col_1_range as usize {
                if let Some(index) = self.selected_sessions.get(i) {
                    if let Some(session) = self.sessions.get(*index) {
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
                            self.open_session = Some(session.name());
                        };
                    };
                };
            }

            // Column 2 gets populated with the remaining buttons
            for i in col_1_range as usize..buttons_total as usize {
                if let Some(index) = self.selected_sessions.get(i) {
                    if let Some(session) = self.sessions.get(*index) {
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
                            self.open_session = Some(session.name());
                        };
                    };
                }
            }
        });
    }

    pub fn session_show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        // What is the currently open session?
        let name = match self.open_session {
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

    fn excercise_type(&self) -> Vec<ExerciseType>;

    /// Entry into the ui codepath.
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts);

    /// Shows the exercise menu.
    fn ui(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut Tts);

    /// Shows the exercise session i.e. the exercise as it is being trained.
    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, tts: &mut Tts);

    // To make sure we can clean up when quitting session.
    fn reset(&mut self);
}
