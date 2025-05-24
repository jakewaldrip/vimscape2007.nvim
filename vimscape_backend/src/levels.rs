use std::collections::HashMap;

use crate::skill_data::SkillData;

pub fn get_updated_levels(
    _skill_data: &Vec<SkillData>,
    _batch_xp: &HashMap<String, i32>,
) -> HashMap<String, i32> {
    // Calculate what level the current xp maps to (using RS level formula)
    // Level formula decides what total xp is required, so will need to derive new one
    let updated_levels: HashMap<String, i32> = HashMap::new();
    updated_levels
}

pub fn get_levels_diff(
    _old_levels: &Vec<SkillData>,
    _new_levels: &HashMap<String, i32>,
) -> HashMap<String, i32> {
    // loop over new levels
    // check for level in old levels, if greater, add to diff
    let levels_diff: HashMap<String, i32> = HashMap::new();
    levels_diff
}

pub fn notify_level_ups(_levels_diff: &HashMap<String, i32>) {
    // figure out finally how to use notify with this (reason we updated crate tbf)
}
