use crate::shared::{AppData, Evaluation};
use crate::widgets::{self, menu_button};
use crate::wm::Exercise;
use chrono::Duration;
use egui::{
    emath::{self, RectTransform},
    epaint::CircleShape,
    pos2, Color32, Pos2, Rect, Response, Sense, Shape,
};
use egui::{vec2, Align, RichText, Vec2};
use rand::prelude::*;

use tts::{self, Tts};

use crate::exercises::shared::grid::Grid;
use crate::exercises::ExerciseStatus;

#[derive(Default)]
struct Answers {
    sequence: Vec<[usize; 2]>,
    response: Vec<[usize; 2]>,
}

/// Sequences
pub struct NumberedSquares {
    seq_length: usize,
    status: ExerciseStatus,
    answers: Answers,
    grid: Grid,
    evaluation: Evaluation<bool>,
}

impl Default for NumberedSquares {
    fn default() -> Self {
        Self {
            answers: Answers::default(),
            seq_length: 4,
            status: ExerciseStatus::None,
            grid: Grid::new(10),
            evaluation: Evaluation::new(Duration::try_seconds(240).unwrap_or_default(), 10),
        }
    }
}

impl NumberedSquares {
    /// Keeps track of exercise progression
    fn progressor(&mut self) {
        // 1. allow response
        // 2. evaluate anwers
        // 3. show result
        // 4. finished

        // end exercise when evaluation is finished.
        if self.evaluation.is_finished() {
            self.status = ExerciseStatus::Finished;
        };
    }

    fn next(&mut self, _: &mut tts::Tts) {
        match self.status {
            ExerciseStatus::Challenge => {
                self.evaluation.add_result(true);
                self.status = ExerciseStatus::Result;
            }
            ExerciseStatus::Result => {
                self.status = ExerciseStatus::Challenge;
            }
            _ => (),
        }
    }

    fn gen_sequence(&mut self) {
        let mut seq = vec![];
        let mut rng = thread_rng();
        while seq.len() < self.seq_length {
            // this means no seq longer than 11 numbers (0..10)!
            let num = rng.gen_range(0..=10);
            if !seq.contains(&num) {
                seq.push(num);
            };
        }
    }

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

    /// Shows the original drawing
    pub fn draw_grid(&mut self, ui: &mut egui::Ui) -> Response {
        // Setup
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );

        // Push shapes to painter
        painter.extend(
            self.grid
                .shapes(self.grid.size(), &to_screen, 1., Color32::KHAKI, true),
        );

        response
    }
}

impl Exercise for NumberedSquares {
    fn name(&self) -> &'static str {
        "Numbered Squares"
    }

    fn description(&self) -> &'static str {
        "Recall the order of the numbered squares."
    }

    fn help(&self) -> &'static str {
        "Todo!"
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
        match self.status {
            ExerciseStatus::None => {
                window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            ExerciseStatus::Finished => {
                window.show(ctx, |ui| self.finished_screen(ui));
            }
            _ => {
                self.progressor();
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
        };

        if self.status != ExerciseStatus::None {}
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
            self.seq_length = i;
            self.evaluation.start();
            self.status = ExerciseStatus::Challenge;
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

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
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

        egui::Frame::canvas(ui.style()).show(ui, |ui| self.draw_grid(ui));
        if self.status == ExerciseStatus::Challenge {}
        if self.status == ExerciseStatus::Response {}
    }
}
