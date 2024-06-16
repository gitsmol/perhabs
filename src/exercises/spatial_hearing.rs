use self::soundsource::{match_coords_to_pin, SoundSource};

use super::ExerciseStage;

use crate::shared::AppData;
use crate::shared::Pos3;
use crate::widgets::evaluation::eval_config_widgets;

use crate::widgets::{self};
use crate::{
    wm::Exercise,
    {shared::Evaluation, shared::Timer},
};
use chrono::Duration;
use egui::{vec2, Align, Frame, Vec2};
use rand::seq::SliceRandom;

mod soundsource;
mod visual_space;

struct SpaceDimensions {
    x: usize,
    y: usize,
    z: usize,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpatialHearing {
    device_url: String,
    status: ExerciseStage,
    space_dimensions: SpaceDimensions,
    sound_sources: Vec<SoundSource>,
    answer: Option<SoundSource>,   // The right answer is a set of coords
    response: Option<SoundSource>, // A soundsource contains a set of coords
    timer: Timer,                  // A timer is useful for all kinds of things
    evaluation: Evaluation<bool>,
}

impl Default for SpatialHearing {
    fn default() -> Self {
        let mut _self = Self {
            device_url: String::from("http://192.168.1.38:5000/buzz"),
            status: ExerciseStage::None,
            space_dimensions: SpaceDimensions { x: 2, y: 3, z: 1 },
            sound_sources: Vec::new(),
            answer: None,
            response: None,
            timer: Timer::new(),
            evaluation: Evaluation::new(Duration::try_seconds(60).unwrap_or_default(), 60),
        };

        // Always initialize default soundsources
        _self.init_soundsources();

        _self
    }
}

// ***********
// Internals: painting, calculations etc
// ***********
impl SpatialHearing {
    /// Initialize the exercise by creating soundsources according to
    /// the parameters in `space_dimensions`.
    fn init_soundsources(&mut self) {
        // We iterate through each dimension in turn, finally constructing an
        // object that contains both the coordinates according to the xyz-system
        // and the normalized Pos3 object.
        // The square size offset determines how far off-center the left top and
        // right bottom of the square are.
        let mut sound_sources: Vec<SoundSource> = Vec::new();
        let x_ratio = 2.0 / self.space_dimensions.x as f32;
        let y_ratio = 2.0 / self.space_dimensions.y as f32;
        let z_ratio = 1.0 / self.space_dimensions.z as f32;
        let x_min = -1.0 + x_ratio / 2.;
        let y_min = -1.0 + y_ratio / 2.;
        for x in 0..self.space_dimensions.x {
            for y in 0..self.space_dimensions.y {
                for z in 0..self.space_dimensions.z {
                    let coords = [x, y, z];
                    let pos3 = Pos3::new(
                        x_min + x as f32 * x_ratio,
                        y_min + y as f32 * y_ratio,
                        z as f32 * z_ratio,
                    );
                    let rect = None;
                    sound_sources.push(SoundSource { coords, pos3, rect })
                }
            }
        }
        self.sound_sources = sound_sources;
    }

    /// Keeps track of answer, response, result progression.
    /// This exercise is only every in Response mode:
    /// - constantly display new arrows until timeout or user input
    /// - record responses:
    ///   - correct response = result 1.0
    ///   - incorrect or no response = result 0.0
    fn progressor(&mut self, ctx: &egui::Context) {
        // Repaint regularly to update timers!
        // NB this also sets bounds on the timer precision.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // When the evaluation time is up or number of reps is reached, stop immediately.
        if self.evaluation.is_finished() {
            self.status = ExerciseStage::Finished;
        }

        match self.status {
            // When we aren't in an exercise, the menu screen is painted by `self.show`.
            ExerciseStage::None => (),
            // Give the challenge (ie make a sound)
            ExerciseStage::Challenge => {
                // Wait for one second before making a sound.
                if !self.timer.is_running() {
                    debug!("SpatialHearing: Setting pre-challenge timer for 1 second.");
                    self.timer.set(Duration::try_seconds(1).unwrap_or_default());
                }
                if !self.timer.is_finished() {
                    return;
                }

                // If we get past the timer guard clause, we proceed with the challenge.
                debug!("SpatialHearing: Pre-challenge timer finished.");

                // Pick a random soundsource
                let mut rng = rand::thread_rng();
                if let Some(sourcesource) = self.sound_sources.choose(&mut rng) {
                    self.answer = Some(sourcesource.to_owned());
                };

                // If an answer is present (ie picking one succeeded), make a sound
                // and progress the exercise.
                let Some(answer) = &self.answer else { return };
                if let Some(pin) = match_coords_to_pin(answer.coords) {
                    debug!(
                        "SpatialHearing: Requesting beep from {} on pin {pin}.",
                        &self.device_url
                    );
                    self.request_beep(&self.device_url, pin, 100, 200);
                }
                self.status = ExerciseStage::Response;
                self.timer.reset();
            }
            // Wait for a response
            ExerciseStage::Response => {
                // If we receive a response, show the result.
                if self.response.is_some() {
                    debug!("SpatialHearing: User response received.");
                    // Evaluate response and store result
                    self.evaluation.add_result(self.evaluate_response());
                    // Move to next step
                    self.status = ExerciseStage::Result
                };
            }
            // Moving from result to next challenge is taken care of in the ui.
            ExerciseStage::Result => {
                // Show result for 2 secs
                if !self.timer.is_running() {
                    debug!("SpatialHearing: Showing result. Setting timer for 2 seconds.");
                    self.timer.set(Duration::try_seconds(2).unwrap_or_default());
                }
                // Then move to the next challenge.
                if self.timer.is_finished() {
                    debug!("SpatialHearing: Result timer finished. Moving to next challenge.");
                    self.status = ExerciseStage::Challenge;

                    // Clean up
                    self.timer.reset();
                    self.response = None;
                }
            }
            // When we are finished, the finished screen is painted by `self.show`.
            ExerciseStage::Finished => (),
        };
    }

    /// Determine correctness of response.
    fn evaluate_response(&self) -> bool {
        // If either answer or response is not available, return false
        let Some(answer) = &self.answer else {
            return false;
        };
        let Some(response) = &self.response else {
            return false;
        };

        debug!("SpatialHearing: Comparing answer and response.");
        debug!("SpatialHearing: answer: {}", answer);
        debug!("SpatialHearing: response: {}", response);

        // Only return 1.0 (correct) if current response matches current answer.
        if answer.coords == response.coords {
            debug!("SpatialHearing: Answer matches response!");
            return true;
        }

        // default answer is false
        false
    }

    /// Send a get request to the ESP32 to trigger a beep.
    fn request_beep(&self, url: &String, pin: usize, freq: usize, sleep_ms: usize) {
        let rq_url = format!("{url}?pin={pin}&freq={freq}&sleep_ms={sleep_ms}");
        let rq = ehttp::Request::get(rq_url);
        ehttp::fetch(rq, |response| match response {
            Ok(res) => debug!("{:?}", res),
            Err(e) => {
                warn!("{:?}", e)
            }
        });
    }
}

// ***********
// UI
// ***********
impl SpatialHearing {
    /// Basic controls during a session
    fn ui_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                // Reset the whole exercise.
                self.reset();
            };
            ui.label(format!(
                "Time remaining: {}:{:02}",
                self.evaluation.time_remaining().num_minutes(),
                self.evaluation.time_remaining().num_seconds()
            ));
            ui.label(format!(
                "Reps remaining: {}",
                self.evaluation.reps_remaining()
            ));
        });
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
}

impl Exercise for SpatialHearing {
    fn name(&self) -> &'static str {
        "Spatial Hearing"
    }

    fn description(&self) -> &'static str {
        "Learn to recognize what direction a sound came from."
    }

    fn reset(&mut self) {
        *self = Default::default();
    }

    fn help(&self) -> &'static str {
        ""
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut tts::Tts) {
        let menu_window = egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(500., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false);

        // Show the right UI, depending on the exercise status.
        match self.status {
            // If we haven't started, show the menu
            ExerciseStage::None => {
                menu_window.show(ctx, |ui| self.ui(ui, appdata, tts));
            }
            // After an exercise, show the review
            ExerciseStage::Finished => {
                menu_window.show(ctx, |ui| self.finished_screen(ui));
            }
            // Any other status means we are in session.
            _ => {
                // Keep track of progression of session
                self.progressor(ctx);
                // Show session panel
                egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {
        // Display the evaluation config
        eval_config_widgets(
            ui,
            &mut self.evaluation.duration,
            &mut self.evaluation.repetitions,
            [30, 300],
            [10, 50],
        );

        // Display spatial setup config on dark background
        ui.set_min_height(300.);
        Frame::dark_canvas(ui.style()).show(ui, |ui| self.draw_3d_space(ui));

        egui::Grid::new("space_dimensions")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Width (x axis)");
                if ui
                    .add(egui::Slider::new(&mut self.space_dimensions.x, 1..=5))
                    .changed()
                {
                    self.init_soundsources()
                };
                ui.end_row();
                ui.label("Height (Y axis)");
                if ui
                    .add(egui::Slider::new(&mut self.space_dimensions.y, 1..=5))
                    .changed()
                {
                    self.init_soundsources()
                };
                ui.end_row();
                ui.label("Depth (Z axis)");
                if ui
                    .add(egui::Slider::new(&mut self.space_dimensions.z, 1..=5))
                    .changed()
                {
                    self.init_soundsources()
                };
                ui.end_row();
            });

        //
        // DEBUGGING STUFF! REMOVE ME!
        //
        if ui.button("Buzz").clicked() {
            self.request_beep(&self.device_url, 12, 200, 200);
        }
        if ui.button("Start session").clicked() {
            self.init_soundsources();
            self.evaluation.start();
            self.status = ExerciseStage::Challenge;
        }
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut tts::Tts) {
        self.ui_controls(ui);

        ui.horizontal(|ui| {
            if let Some(answer) = &self.answer {
                ui.label(format!("answer: {}", answer));
            };
            if let Some(response) = &self.response {
                ui.label(format!("resp: {}", response));
            }
        });

        Frame::dark_canvas(ui.style()).show(ui, |ui| {
            // Draw the '3d' space and fill it with the configured soundsources.
            let visual_space = self.draw_3d_space(ui);
            // Taking care of clicks while we are in response mode
            if self.status == ExerciseStage::Response && visual_space.clicked() {
                // On click, get pointer position. If user clicks a soundsource,
                // store that answer for processing.
                if let Some(pointer_pos) = visual_space.interact_pointer_pos() {
                    // Find a soundsource that matches the click.
                    let matching_source = self.sound_sources.iter().find(|s| {
                        let Some(rect) = s.rect else { return false };
                        if rect.contains(pointer_pos) {
                            debug!("SpatialHearing: matched click to {s}");
                            return true;
                        }
                        false
                    });

                    // If a matching source is found, progress the exercise.
                    if let Some(source) = matching_source {
                        self.response = Some(source.clone());
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use egui::Context;

    use crate::{
        exercises::{spatial_hearing::soundsource::SoundSource, ExerciseStage},
        shared::Pos3,
    };

    use super::SpatialHearing;

    #[test]
    fn single_progression() {
        // create the exercise
        let mut exercise = SpatialHearing::default();
        let mut ctx = Context::default();

        // start the exercise
        exercise.init_soundsources();
        exercise.status = ExerciseStage::Challenge;
        exercise.progressor(&mut ctx);
        sleep(Duration::from_secs(1));
        exercise.progressor(&mut ctx);

        // In response mode, create a response
        assert!(exercise.status == ExerciseStage::Response);
        exercise.response = Some(SoundSource {
            coords: [9, 9, 9],
            pos3: Pos3::new(1., 1., 1.),
            rect: None,
        });
        exercise.progressor(&mut ctx);

        // In result mode, wait 2 seconds and go to next challenge
        assert!(exercise.status == ExerciseStage::Result);
        exercise.progressor(&mut ctx);
        sleep(Duration::from_secs(2));
        exercise.progressor(&mut ctx);
        assert!(exercise.status == ExerciseStage::Challenge);

        // assert_eq!(score, 0.59999996);
    }
}
