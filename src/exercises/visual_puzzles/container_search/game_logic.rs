use super::ContainerSearch;
use crate::exercises::ExerciseStage;
use chrono::Duration;
use egui::Pos2;

impl ContainerSearch {
    /// Keeps track of exercise progression
    pub(super) fn progressor(&mut self) {
        // end exercise when evaluation is finished.
        if self.evaluation.is_finished() {
            self.stage = ExerciseStage::Finished;
        };

        // if we are still running, progress through exercise stages
        match self.stage {
            // Showing correct/incorrect for current move
            ExerciseStage::Result => {
                // We are showing the last result from `self.round_scores` until the timer is finished.
                if self.result_timer.is_finished() {
                    // Now we must determine the outcome.
                    // If we are finished, go next challenge and up level.
                    if self.containers.found_secrets.len() == self.num_containers {
                        self.evaluation.add_result((self.num_containers, true));
                        self.num_containers += 1;
                        self.next_challenge();
                        return;
                    }

                    // If we have an incorrect answer, go next challenge.
                    if let Some(result) = self.round_score.last() {
                        if result == &false {
                            self.evaluation.add_result((self.num_containers, false));

                            // If this is the second fail, reduce level.
                            if let Some(last_two_results) =
                                self.evaluation.show_results().last_chunk::<2>()
                            {
                                if last_two_results[0].1 == false && last_two_results[1].1 == false
                                {
                                    self.num_containers -= 1;
                                }
                            };

                            self.next_challenge();
                            return;
                        }
                    }

                    // Else, just go next round.
                    self.next_round();
                }
            }
            _ => (),
        };
    }

    /// Evaluate response, store result, move on to next move
    fn next_round(&mut self) {
        self.containers.reset_opened();
        self.gen_secret();
        self.stage = ExerciseStage::Response;
    }

    /// Evaluate response, store result, move on to next challenge
    fn next_challenge(&mut self) {
        self.round_score.clear();
        self.containers.clear();
        self.gen_containers();
        self.gen_secret();
        self.stage = ExerciseStage::Response;
    }

    /// Evaluate response:
    /// - does it match the secret? Success and next move. Adjust difficulty.
    /// - does it match a previous secret? Fail and next challenge. Adjust difficulty.
    pub(super) fn evaluate_response(&mut self, pos: &Pos2) {
        // Pos isn't a container: return
        if !self.containers.unopened.contains(pos) {
            return;
        }

        // This container contains an already found secret: fail
        if self.containers.found_secrets.contains(pos) {
            self.evaluation.add_result((self.num_containers, false));
            self.stage = ExerciseStage::Result;
            self.result_timer
                .set(Duration::try_milliseconds(self.result_ms).unwrap_or_default());
            return;
        };

        // This container contains the secret: next
        if self.containers.secret == *pos {
            self.containers.found_secrets.push(*pos);
            self.round_score.push(true);

            // This is the last container with a secret: succes
            if self.containers.found_secrets.len() == self.num_containers {
                self.evaluation.add_result((self.num_containers, true));
            }
            self.stage = ExerciseStage::Result;
            self.result_timer
                .set(Duration::try_milliseconds(self.result_ms).unwrap_or_default());
            return;
        };

        // We opened an empty container: keep going
        self.containers.unopened.retain(|f| f != pos);
        self.containers.opened.push(*pos);
    }
}
