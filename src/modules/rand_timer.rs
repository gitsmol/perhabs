use crate::asset_loader::AppData;
use crate::windowman::{AppWin, View};
use egui::RichText;
use rand::prelude::*;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep, JoinHandle};
use std::time::Duration;
use tts::{self, Tts};

/// RandTimer runs a loop. Within this 'outer' loop, short delays of a random duration
/// are set. When a delay runs out, 'switch' is spoken by TTS. Within the outer thread,
/// an inner thread runs a timer. When the timer runs out, 'finished' is spoken by TTS.
pub struct RandTimer {
    timer_mins: u64,
    min_secs: u64,
    max_secs: u64,
    session_thread: Option<JoinHandle<()>>,
    session_active: bool,
    act: Arc<Mutex<bool>>,
    session_tx: mpsc::Sender<u64>,
    session_rx: mpsc::Receiver<u64>,
    remaining_secs: u64,
}

impl Default for RandTimer {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            timer_mins: 10,
            min_secs: 60,
            max_secs: 120,
            session_thread: None,
            session_active: false,
            act: Arc::new(Mutex::new(false)),
            session_tx: tx,
            session_rx: rx,
            remaining_secs: 0,
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

        // This tx is used to set session state after session ends.
        let tx_session = self.session_tx.clone();

        let session_active = self.act.clone();
        // Spawn outer thread so we don't block UI updates
        let outer = thread::spawn(move || {
            // Spawn inner thread to keep track of timer
            let (tx_timer, rx_timer) = mpsc::channel();

            thread::spawn(move || {
                let secs = timer_mins * 60;
                debug!("Spawning inner timer for {} secs", secs);
                for i in (0..=secs).rev() {
                    // Is the session still active?
                    let session = session_active.lock().unwrap();
                    debug!("Session mutex unlocked.");
                    if *session == false {
                        info!("Stopping inner thread.");
                        break;
                    }
                    sleep(Duration::from_secs(1));
                    match tx_session.send(i) {
                        Ok(_) => debug!("Updating remaining seconds."),
                        Err(_) => warn!("Couldn't update remaining seconds."),
                    };
                }
                sleep(Duration::from_secs(secs));
                match tx_timer.send(true) {
                    Ok(_) => info!("Ending outer loop."),
                    Err(_) => info!("Couldn't end outer loop."),
                };
            });

            // This is the main loop.
            // Unless we receive a message from the 'inner' time keeping thread,
            // keep creating delays of a random amount of time and then say switch.
            let mut rng = thread_rng();
            loop {
                // Did we get a message from the 'inner' timer?
                if let Ok(_) = rx_timer.try_recv() {
                    debug!("Received stop signal from outer loop.");
                    spk.speak("Finished!", true).unwrap_or_default();
                    // sleep for a couple secs here so the thread isn't killed
                    // before speech is finished
                    sleep(Duration::from_secs(5));
                    break;
                }
                let secs = rng.gen_range(min_secs..=max_secs);
                debug!("Short outer delay: waiting for {} secs.", secs);
                sleep(Duration::from_secs(secs));
                spk.speak("Switch", true).unwrap_or_default();
            }
        });
        self.session_thread = Some(outer);
    }
}

impl AppWin for RandTimer {
    fn name(&self) -> &'static str {
        "Timer"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, appdata: &AppData, tts: &mut Tts) {
        // update the session info

        match self.session_rx.try_recv() {
            Ok(res) => self.remaining_secs = res,
            Err(_) => (),
        }

        if self.session_active {
            // Always repaint when in session. This is necessary because egui by default
            // does not run this function if there is no input. If we don't request repaint
            // none of the logic in session is run.
            ctx.request_repaint();
            egui::CentralPanel::default().show(ctx, |ui| self.session(ui, appdata, tts));
        }
        if !self.session_active {
            egui::Window::new(self.name())
                .open(open)
                .default_height(500.0)
                .show(ctx, |ui| self.ui(ui, appdata, tts));
        }
    }
}

impl View for RandTimer {
    fn ui(&mut self, ui: &mut egui::Ui, _: &AppData, tts: &mut Tts) {
        // basic configuration UI
        ui.vertical(|ui| {
            ui.add(egui::Slider::new(&mut self.timer_mins, 1..=30).suffix("min"));
            ui.add(egui::Slider::new(&mut self.min_secs, 30..=120).suffix("sec"));
            ui.add(egui::Slider::new(&mut self.max_secs, 90..=300).suffix("sec"));
            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    self.session_active = true;
                    *self.act.lock().unwrap() = true;
                    self.run(tts)
                }
            });
        });
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        ui.horizontal(|ui| {
            if ui.button("Stop").clicked() {
                *self.act.lock().unwrap() = false;
                self.session_active = false;
                debug!("Session mutex unlocked and set to false.");
            }
        });

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 4.);
            ui.label("Time remaining");
            ui.heading(RichText::new(format!("{}", &self.remaining_secs)).size(25.));
            ui.add_space(20.);
        });
    }
}
