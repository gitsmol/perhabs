use crate::{shared::asset_loader::appdata::AppData, wm::sessionman::Exercise};
use egui::{vec2, Align, Vec2};
use rand::prelude::*;
use tts::Tts;

pub struct NumSeq {
    operators: Vec<i32>,
    sequence: Vec<i32>,
    seq_length: u32,
    operators_num: usize,
    operators_var: i32,
    seq_show: bool,
}

impl NumSeq {
    fn gen_seq(&mut self) -> () {
        self.sequence.resize(1, 0);
        self.operators.resize(0, 0);
        let mut rng = thread_rng();
        while self.operators.len() < self.operators_num {
            let sign = if self.operators.len() % 2 == 0 { 1 } else { -1 };
            let op = sign * rng.gen_range(1..=self.operators_var);
            if !self.operators.contains(&op) {
                self.operators.push(op);
            }
        }
        for _ in 1..self.seq_length {
            for step in &self.operators {
                // length starts from 1, index starts from 0.
                let lastnum = self.sequence[self.sequence.len() - 1];
                let num = lastnum + step;
                self.sequence.push(num);
            }
        }
        // for _ in self.operators.iter() {}
    }
}
impl Default for NumSeq {
    fn default() -> Self {
        Self {
            operators: vec![8, -6, 1],
            sequence: vec![0],
            seq_length: 5,
            operators_num: 2,
            operators_var: 10,
            seq_show: false,
        }
    }
}

impl Exercise for NumSeq {
    fn name(&self) -> &'static str {
        "Number sequences"
    }

    fn description(&self) -> &'static str {
        "Calculate numbers based on simple rules."
    }

    fn reset(&mut self) {
        *self = NumSeq::default();
    }

    fn show(&mut self, ctx: &egui::Context, appdata: &AppData, tts: &mut Tts) {
        egui::Window::new(self.name())
            .anchor(
                egui::Align2([Align::Center, Align::TOP]),
                Vec2::new(0., 100.),
            )
            .fixed_size(vec2(350., 300.))
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .show(ctx, |ui| self.ui(ui, appdata, tts));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _appdata: &AppData, _: &mut Tts) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button("Generate").clicked() {
                    self.gen_seq();
                }
                if ui.button("Clear").clicked() {
                    self.sequence.clear();
                }
                ui.checkbox(&mut self.seq_show, "Show sequence");
            });
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Operators");
                    ui.add(egui::Slider::new(&mut self.operators_num, 0..=5));
                    ui.end_row();

                    ui.label("Operator variance");
                    ui.add(egui::Slider::new(&mut self.operators_var, 0..=20));
                    ui.end_row();

                    ui.label("Sequence length");
                    ui.add(egui::Slider::new(&mut self.seq_length, 0..=25));
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Operators");
                    ui.heading(format!("{:?}", &self.operators));
                    ui.end_row();
                });
            if self.seq_show {
                ui.separator();
                ui.heading(format!("{:?}", &self.sequence));
            }
        });
    }

    fn session(&mut self, ui: &mut egui::Ui, _: &AppData, _: &mut Tts) {
        todo!()
    }
}
