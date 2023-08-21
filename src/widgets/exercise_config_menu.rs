use super::menu_button;
use crate::shared::asset_loader::exercise_config::ExerciseConfig;

/// Draws a menu consisting of two columns. All menu items are distributed across
/// a given number of columns. For an uneven number of items,
/// the last column gets the smallest number of items.
///
/// To use, define T using turbofish (::<>) and load a vec containing exercise configs.
/// When a button is clicked, it returns a reference to a specific config from this vec.
/// The reference is wrapped in Option, so the function returns None when not clicked.
///
/// ## Example
/// ```ignore
/// if let Some(list_of_configs) =
///     exercise_config_menu::<DepthPerceptionConfig>(&mut ui, &config.depth_perception, 3)
/// {
///     let mut return_val: Option<&T> = None;
///     for config in list_of_configs {
///         if ui.button("Menu option").clicked {
///             return_val = Some(config);
///         };
///     };
///
///     return_val
/// };
/// ```
pub fn exercise_config_menu_multicol<'a, T>(
    ui: &mut egui::Ui,
    config: &'a Vec<T>,
    num_cols: usize,
) -> Option<&'a T>
where
    T: ExerciseConfig,
{
    // We return None unless a button gets clicked.
    let mut return_val: Option<&T> = None;

    // Divide buttons across cols. For an uneven number of buttons, we want the least
    // amount of buttons in the last column. Therefore, we use ceil() to get the highest
    // possible number from the division.
    let total_items = config.len();
    let bin_size = (total_items as f32 / num_cols as f32).ceil() as usize;

    // Each column is filled with `bin_size` buttons so long as an item can be
    // found in the config file for that index. If not, no button is created.
    ui.columns(num_cols, |col| {
        for colnr in 0..num_cols {
            let item_index_start = colnr * bin_size;
            let item_index_end = colnr * bin_size + bin_size;
            for i in item_index_start..item_index_end {
                if let Some(exercise) = config.get(i) {
                    if menu_button(&mut col[colnr], None, exercise.name(), "").clicked() {
                        return_val = Some(exercise);
                    };
                };
            }
        }
    });

    return_val
}
