use super::menu_button;
use crate::shared::asset_loader::exercise_config::ExerciseConfig;

/// Draws a menu consisting of two columns. All menu items are distributed evenly
/// across the two columns, filling the first and then the second. For an uneven number
/// of items, the extra item goes into the first column.
///
/// To use, define T using turbofish (::<>) and load a vec containing exercise configs.
/// When a button is clicked, it returns a reference to a specific config from this vec.
/// The reference is wrapped in an option, so the function returns None when not clicked.
///
/// ## Example
/// ```ignore
/// if let Some(config) =
///     exercise_config_menu::<DepthPerceptionConfig>(&mut ui, &config.depth_perception)
/// {
///    println!("Config {} has been clicked!", config.name());
/// };
/// ```

pub fn exercise_config_menu<'a, T>(ui: &mut egui::Ui, config: &'a Vec<T>) -> Option<&'a T>
where
    T: ExerciseConfig,
{
    let buttons_total: f32 = config.len() as f32;
    let col_1_range = buttons_total - (buttons_total / 2.).floor();

    let mut return_val: Option<&T> = None;

    ui.columns(2, |col| {
        // Column 1 gets populated with at least half the buttons
        for i in 0..col_1_range as usize {
            if let Some(exercise) = config.get(i) {
                if menu_button(&mut col[0], None, exercise.name(), "").clicked() {
                    return_val = Some(exercise);
                };
            };
        }

        // Column 2 gets populated with the remaining buttons
        for i in col_1_range as usize..buttons_total as usize {
            if let Some(exercise) = config.get(i) {
                if menu_button(&mut col[1], None, exercise.name(), "").clicked() {
                    return_val = Some(exercise);
                };
            };
        }
    });

    return_val
}
