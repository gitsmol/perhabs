use egui::Pos2;
use rand::prelude::*;

use super::ContainerSearch;

#[derive(Default)]
pub struct Containers {
    pub secret: Pos2,
    pub unopened: Vec<Pos2>,
    pub found_secrets: Vec<Pos2>,
    pub opened: Vec<Pos2>,
}

impl Containers {
    pub fn clear(&mut self) {
        self.secret = Pos2::ZERO;
        self.unopened.clear();
        self.found_secrets.clear();
        self.opened.clear();
    }

    pub fn reset_opened(&mut self) {
        self.unopened
            .extend(self.opened.drain(..).collect::<Vec<Pos2>>());
    }
}

impl ContainerSearch {
    /// Generates a sequence of valid and unique positions on the grid.
    pub(super) fn gen_containers(&mut self) {
        self.containers.clear();
        let mut rng = thread_rng();
        let all_coords: Vec<Pos2> = self
            .grid
            .get_all_coords(self.grid_size)
            .into_iter()
            .flatten()
            .collect();

        while self.containers.unopened.len() < self.num_containers {
            let num = rng.gen_range(0..all_coords.len());
            if let Some(pos) = all_coords.get(num) {
                if !self.containers.unopened.contains(pos) {
                    self.containers.unopened.push(*pos);
                };
            };
        }
    }

    /// Selects one container as containing the secret
    pub fn gen_secret(&mut self) {
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 1000;

        while attempts < MAX_ATTEMPTS {
            if let Some(secret) = self.containers.unopened.choose(&mut rng) {
                if !self.containers.found_secrets.contains(secret) {
                    self.containers.secret = *secret;
                    return;
                }
            }
            attempts += 1;
        }
    }
}
