use crate::modules::cpal_audio::*;
use std::sync::mpsc::{self, Sender};

use crate::modules::asset_loader::AppData;
use crate::wm::windowman::{AppWin, View};

use chrono::Duration;
use tts::{self, Tts};

use super::timer::Timer;

pub struct Metronome {
    running: bool,
    bpm: i64,
    audiohandle: Option<AudioHandle>,
    voice: Voice,
    voice_tx: Sender<Voice>,
    beat_timer: Timer,
}

impl Default for Metronome {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            running: false,
            bpm: 60,
            audiohandle: None,
            voice: Voice::default(),
            voice_tx: tx, // This does nothing but i didn't want to wrap this in option
            beat_timer: Timer::new(),
        }
    }
}

impl Metronome {
    fn init_audio(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.voice_tx = tx;
        let audiohandle = beep(rx);
        // audiohandle.0.pause();
        self.audiohandle = Some(audiohandle);
    }

    fn metronome_loop(&mut self) {
        if self.beat_timer.is_finished() {
            self.voice_tx.send(self.voice);
            let bpm_millis: i64 = (60 / self.bpm) * 1000;
            self.beat_timer.set(Duration::milliseconds(bpm_millis));
        }
    }
}

impl AppWin for Metronome {
    fn name(&self) -> &'static str {
        "Metronome"
    }

    // main loop
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, appdata: &AppData, tts: &mut Tts) {
        if self.running {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            self.metronome_loop();
        }
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, appdata, tts));
    }
}

impl View for Metronome {
    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        // basic configuration UI
        ui.vertical(|ui| {
            ui.add(egui::Slider::new(&mut self.bpm, 46..=90).suffix("BPM"));

            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    // On WASM, we have to reinitialize the audiohandle every time we
                    // want to play the stream. Probably due to the audio playing on the
                    // main thread in wasm.
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.init_audio();
                        self.voice_tx.send(self.voice);
                    }
                    // Other platforms don't have this problem because they give the
                    // audio stream its own thread.
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(handle) = &self.audiohandle {
                        handle.play();
                    } else {
                        self.init_audio();
                    }
                }

                if ui.button("Stop").clicked() {
                    if let Some(handle) = &self.audiohandle {
                        handle.pause();
                    }
                }

                if ui.button("Add voice").clicked() {
                    self.voice_tx.send(self.voice);
                }
            });
            ui.add(egui::Slider::new(&mut self.voice.freq, 80.0..=20_000.0).text("Freq"));
            ui.add(egui::Slider::new(&mut self.voice.attack, 1000..=30000).text("Attack"));
            ui.add(egui::Slider::new(&mut self.voice.sustain, 1000..=30000).text("Sustain"));
            ui.add(egui::Slider::new(&mut self.voice.release, 1000..=30000).text("Release"));

            if ui.button("Start metronome").clicked() {
                self.running = true;
            }
            if ui.button("Stop metronome").clicked() {
                self.running = false;
            }
        });
    }
}
