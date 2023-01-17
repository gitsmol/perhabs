use crate::windowman::{AppWin, View};
use fastrand;

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
        while self.operators.len() < self.operators_num {
            let sign = if self.operators.len() % 2 == 0 { 1 } else { -1 };
            let op = sign * fastrand::i32(1..=self.operators_var);
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
        for _ in self.operators.iter() {}
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

impl AppWin for NumSeq {
    fn name(&self) -> &'static str {
        "Number sequences"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, mut spk: &mut tts::Tts) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui, &mut spk));
    }
}

impl View for NumSeq {
    fn ui(&mut self, ui: &mut egui::Ui, spk: &mut tts::Tts) {
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
}
