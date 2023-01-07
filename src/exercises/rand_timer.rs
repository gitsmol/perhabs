use crate::windowman::{AppWin, View};
use fastrand;
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;
use tts;

pub struct RandTimer {
    timer_mins: u64,
    min_secs: u64,
    max_secs: u64,
}

impl Default for RandTimer {
    fn default() -> Self {
        Self {
            timer_mins: 10,
            min_secs: 60,
            max_secs: 120,
        }
    }
}

impl RandTimer {
    fn run(&mut self, spk: &mut tts::Tts) -> () {
        // clone values to move into thread
        let timer_mins = self.timer_mins.clone();
        let min_secs = self.min_secs.clone();
        let max_secs = self.max_secs.clone();
        let mut spk = spk.clone();

        thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let secs = timer_mins * 60;
                debug!("Spawning timer for {} secs", secs);
                sleep(Duration::from_secs(secs));
                match tx.send(true) {
                    Ok(_) => info!("Ending timer loop."),
                    Err(_) => info!("Couldn't end timer loop."),
                };
            });

            loop {
                if let Ok(_) = rx.try_recv() {
                    debug!("Received stop signal.");
                    spk.speak("Finished!", true).unwrap_or_default();
                    return;
                }
                let secs = fastrand::u64(min_secs..=max_secs);
                debug!("Loop: waiting for {} secs.", secs);
                sleep(Duration::from_secs(secs));
                spk.speak("Switch", true).unwrap_or_default();
            }
        });
    }
}

impl AppWin for RandTimer {
    fn name(&self) -> &'static str {
        "Timer"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, mut spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, &mut spk));
    }
}

impl View for RandTimer {
    fn ui(&mut self, ui: &mut egui::Ui, spk: &mut tts::Tts) {
        // self.say(spk);

        // normal stuff
        ui.vertical(|ui| {
            ui.add(egui::Slider::new(&mut self.timer_mins, 1..=30).suffix("min"));
            ui.add(egui::Slider::new(&mut self.min_secs, 30..=120).suffix("sec"));
            ui.add(egui::Slider::new(&mut self.max_secs, 90..=300).suffix("sec"));
            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    self.run(spk)
                }
                if ui.button("Stop").clicked() {}
            });
        });
    }
}
