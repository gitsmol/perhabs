use std::iter::zip;

use chrono::Duration;
use egui::*;

use super::{
    circle_mutable::{circle_input_duration, circle_input_integer},
    circle_with_data,
};

/// Review the evaluation.
pub fn post_eval_widgets(
    ui: &mut egui::Ui,
    total_score: f32,
    reps_done: usize,
    time_taken: String,
) {
    // Format total average score.
    let total_score_color = match total_score {
        x if x > 0.8 => Color32::GREEN,
        x if x > 0.5 => Color32::BLUE,
        _ => Color32::from_rgb(255, 165, 0),
    };
    let total_score_formatted = format!("{:.0}%", total_score * 100.0);

    ui.horizontal(|ui| {
        circle_with_data(
            ui,
            &reps_done.to_string(),
            &String::from("Reps done"),
            100.,
            Color32::BLUE,
        );
        circle_with_data(
            ui,
            &time_taken,
            &String::from("Time taken"),
            100.,
            Color32::BLUE,
        );

        circle_with_data(
            ui,
            &total_score_formatted,
            &String::from("Average score"),
            100.,
            total_score_color,
        );
    });
}

/// A set of widgets to configure the evalation parameters.
pub fn eval_config_widgets(ui: &mut egui::Ui, duration: &mut Duration, reps: &mut usize) {
    ui.heading("Session length");
    ui.separator();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_size(vec2(225., 100.));
        ui.label("Pick a session length or manually change duration and repetitions. The session ends when time runs out or the maximum number of repetitions is reached.");
        ui.add_space(15.);
        ui.horizontal(|ui| {
            for (name, i) in zip(["Short", "Medium", "Long"], [60, 90, 120]) {
                if ui.add_sized(vec2(60., 30.), |ui: &mut egui::Ui| ui.button(name)
                ).clicked() {
                    *duration = Duration::seconds(i);
                    *reps = i as usize;
            };

            };
        });
        });
        ui.add_space(25.);
        ui.vertical(|ui| {

            ui.horizontal(|ui| {
                let duration_circle = circle_input_duration(
                    ui,
                    duration,
                    &Duration::seconds(1),
                    &Duration::seconds(10),
                    &Duration::seconds(120),
                    &String::from("Duration"),
                    125.,
                    Color32::DARK_BLUE,
                );
                if duration_circle.double_clicked() {
                    *duration = Duration::seconds(60);
                }
                let reps_circle = circle_input_integer(
                    ui,
                    reps,
                    &1,
                    &5,
                    &120,
                    &String::from("Repetitions"),
                    125.,
                    Color32::DARK_BLUE,
                );
                if reps_circle.double_clicked() {
                    *reps = 60;
                }
            });

        });
    });
    ui.separator();
}
