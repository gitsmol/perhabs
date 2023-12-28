use chrono::Duration;
use egui::*;

use super::{
    circle_mutable::{circle_mut_duration, circle_mut_integer},
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
///
/// # Example
/// Lets say we have a struct with a field for `duration` and `reps`.
/// This example will present a set of widgets to change both duration and reps within
/// their respective min/max values of 1 - 120 and 1 - 60.
/// ```
/// eval_config_widgets(ui, &mut mystruct.duration, &mut mystruct.reps, [Duration::seconds(1), Duration::seconds(120)], [1, 60]);
/// ```
pub fn eval_config_widgets(
    ui: &mut egui::Ui,
    duration: &mut Duration,
    reps: &mut usize,
    duration_range_secs: [i64; 2],
    rep_range: [usize; 2],
) {
    ui.heading("Session length");
    ui.separator();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            // We need to set a maximum size so the label gets wrapped.
            ui.set_max_size(vec2(225., 100.));
        ui.label("Pick a session length or manually change duration and repetitions. The session ends when time runs out or the maximum number of repetitions is reached.");
        ui.add_space(15.);

        // Add shortcuts to set session length
        ui.horizontal(|ui| {
            if ui.add_sized(vec2(60., 30.), |ui: &mut egui::Ui| ui.button("Short")
            ).clicked() {
                *duration = Duration::seconds(duration_range_secs[0]);
                *reps = rep_range[0];
            };
            if ui.add_sized(vec2(60., 30.), |ui: &mut egui::Ui| ui.button("Medium")
            ).clicked() {
                *duration = Duration::seconds(duration_range_secs[1] / 2);
                *reps = rep_range[1] / 2;
            };
            if ui.add_sized(vec2(60., 30.), |ui: &mut egui::Ui| ui.button("Long")
            ).clicked() {
                *duration = Duration::seconds(duration_range_secs[1]);
                *reps = rep_range[1];
            };
        });
        });
        ui.add_space(25.);

        // Add round widgets
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                circle_mut_duration(
                    ui,
                    duration,
                    &Duration::seconds(1),
                    &Duration::seconds(duration_range_secs[0]),
                    &Duration::seconds(duration_range_secs[1]),
                    &String::from("Duration"),
                    125.,
                    Color32::DARK_BLUE,
                );

                circle_mut_integer(
                    ui,
                    reps,
                    &1,
                    &rep_range.get(0).unwrap_or(&1),
                    &rep_range.get(1).unwrap_or(&60),
                    &String::from("Repetitions"),
                    125.,
                    Color32::DARK_BLUE,
                );

            });

        });
    });
    ui.separator();
}
