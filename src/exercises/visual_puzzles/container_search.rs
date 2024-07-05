use crate::shared::{AppData, Evaluation, Timer};
use crate::widgets::{self, menu_button};
use crate::wm::{Exercise, ExerciseType};
use chrono::Duration;

mod containers;
use containers::Containers;
use egui::{vec2, Align, Vec2};

use tts::{self, Tts};
mod game_logic;
mod session_ui;
use crate::exercises::shared::grid::Grid;
use crate::exercises::ExerciseStage;

/// The user is shown a number of containers. In one of the containers is a secret.
/// The secret is found when the container is opened by clicking on it. The container
/// with the secret is thereafter unopenable. Trying to open this container loses the game.
/// All other containers are closed and the secret is hidden in a random container.
///
/// The player must memorize the secrets they have found so far in order to prevent
/// opening a container with a previously found secret.
pub struct ContainerSearch {
    stage: ExerciseStage,
    num_containers: usize, // basic difficulty setting
    containers: Containers,
    grid: Grid,
    grid_size: usize,
    result_timer: Timer,
    result_ms: i64,
    round_score: Vec<bool>,
    evaluation: Evaluation<(usize, bool)>,
}

impl Default for ContainerSearch {
    fn default() -> Self {
        Self {
            containers: Containers::default(),
            num_containers: 4,
            stage: ExerciseStage::None,
            grid: Grid::new(),
            grid_size: 10,
            result_timer: Timer::new(),
            result_ms: 1000,
            round_score: Vec::new(),
            evaluation: Evaluation::new(Duration::try_seconds(240).unwrap_or_default(), 10),
        }
    }
}

impl ContainerSearch {
    /// Review the evaluation.
    fn finished_screen(&mut self, ui: &mut egui::Ui) {
        widgets::evaluation::post_eval_widgets(
            ui,
            self.evaluation.average_score(),
            self.evaluation.reps_done(),
            self.evaluation.time_taken_as_string(),
        );

        // Close
        if ui.button("Close").clicked() {
            self.reset();
        }
    }

    fn draw_debug_info(&self, ui: &mut egui::Ui) {
        // debug info in top bar
        ui.horizontal(|ui| {
            ui.label(format!("Stage: {:?}", self.stage));
            ui.add_space(10.0);
            ui.label(format!(
                "Found secrets: {:?}",
                self.containers.found_secrets
            ));
            ui.label(format!("Secret: {:?}", self.containers.secret));
            ui.label(format!("Opened: {:?}", self.containers.opened));
        });
    }
}

impl Exercise for ContainerSearch {
    fn name(&self) -> &'static str {
        "Containers and secrets"
    }

    fn description(&self) -> &'static str {
        "Find the secret in the containers. Memorize the secrets you've found."
    }

    fn help(&self) -> &'static str {
        "You are presented with a number of boxes. In one of the boxes is a secret. If you find the secret, the boxes close again. It is now forbidden to click the box containing the secret you found. A new secret is hidden in one of the boxes. You must find it! Repeat this until all boxes contain a secret you found."
    }

    fn excercise_type(&self) -> Vec<ExerciseType> {
        vec![ExerciseType::Cognitive, ExerciseType::Visual]
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    /// Show the configuration dialog
    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        // Define menu window
        let window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(350., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        // If we aren't showing the menu or the finished screen, we're in a session.
        match self.stage {
            ExerciseStage::None => {
                window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            ExerciseStage::Finished => {
                window.show(ctx, |ui| self.finished_screen(ui));
            }
            _ => {
                self.progressor();
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
        };

        if self.stage != ExerciseStage::None {}
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        ui.label(self.help());
        ui.separator();

        // Show evaluation config
        widgets::evaluation::eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 600],
            [5, 60],
        );

        // Draw a menu in two columns
        let mut func = |i| {
            self.num_containers = i;
            self.evaluation.start();
            self.gen_containers();
            self.gen_secret();
            self.stage = ExerciseStage::Response;
        };

        ui.columns(2, |col| {
            // Column 1 gets populated with at least half the buttons
            for i in 4..8 as usize {
                if menu_button(&mut col[0], None, None, format!("{i} numbers").as_str(), "")
                    .clicked()
                {
                    func(i);
                };
            }

            // Column 2 gets populated with the remaining buttons
            for i in 8..=10 as usize {
                if menu_button(&mut col[1], None, None, format!("{i} numbers").as_str(), "")
                    .clicked()
                {
                    func(i);
                };
            }
        });
    }

    fn session(&mut self, ui: &mut egui::Ui, appdata: &AppData, _: &mut Tts) {
        // Always check progression
        self.progressor();

        if appdata.debug {
            self.draw_debug_info(ui);
        };

        // session menu bar
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                *self = Default::default();
            };
            ui.label(format!(
                "Time remaining: {}",
                self.evaluation.time_remaining_as_string()
            ));
            ui.label(format!(
                "Reps remaining: {}",
                self.evaluation.reps_remaining()
            ));
        });

        // Draw grid
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            self.draw_session(ui, appdata.debug);
        });
    }
}
