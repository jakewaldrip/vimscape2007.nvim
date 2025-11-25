use std::collections::HashMap;

use nvim_oxi::{
    api::{notify, types::LogLevel},
    Dictionary,
};
use once_cell::sync::Lazy;

use crate::skill_data::SkillData;

const XP_BASE: f32 = 75.0;
const XP_MULTIPLIER: f32 = 1.10409;

static CUMULATIVE_XP: Lazy<[f32; 100]> = Lazy::new(|| {
    let mut xp = [0.0; 100];
    let mut total = 0.0;
    for level in 1usize..=99 {
        let delta = XP_BASE * XP_MULTIPLIER.powi(level as i32);
        total += delta;
        xp[level] = total;
    }
    xp
});

/// Updates skill levels based on batch XP gains.
///
/// Returns a map of skill names to their new levels for skills that have XP in batch_xp.
pub fn get_updated_levels(
    skill_data: &[SkillData],
    batch_xp: &HashMap<String, i32>,
) -> HashMap<String, i32> {
    let mut updated_levels: HashMap<String, i32> = HashMap::new();

    for data in skill_data {
        if let Some(exp) = batch_xp.get(&data.skill_name) {
            let new_level = get_level_for_exp(*exp);
            updated_levels.insert(data.skill_name.clone(), new_level);
        }
    }
    updated_levels
}

/// Calculates the skill level for a given amount of experience points.
///
/// The level is determined based on the cumulative XP required, using the formula:
/// XP to next level = 75 * 1.10409 ^ level
fn get_level_for_exp(exp: i32) -> i32 {
    if exp < 0 {
        return 1;
    }
    let exp_f = exp as f32;
    let idx = CUMULATIVE_XP.partition_point(|&c| c <= exp_f);
    (idx as i32).max(1).min(99)
}

/// Computes the difference in levels, returning only skills that have leveled up.
///
/// Returns a map of skill names to their new levels where new_level > old_level.
pub fn get_levels_diff(
    skill_data: &[SkillData],
    new_levels: &HashMap<String, i32>,
) -> HashMap<String, i32> {
    let mut levels_diff: HashMap<String, i32> = HashMap::new();
    for old_data in skill_data {
        if let Some(new_level) = new_levels.get(&old_data.skill_name) {
            if new_level > &old_data.level {
                levels_diff.insert(old_data.skill_name.clone(), *new_level);
            }
        }
    }
    levels_diff
}

/// Notifies about level-ups via Neovim's notification system.
pub fn notify_level_ups(levels_diff: &HashMap<String, i32>) {
    let notify_opts = Dictionary::new();
    for level_data in levels_diff {
        let skill_name = level_data.0;
        let level = level_data.1;
        if let Err(e) = notify(
            &format!("{skill_name} reached level {level}!"),
            LogLevel::Info,
            &notify_opts,
        ) {
            eprintln!("Failed to notify level up for {skill_name}: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_level_for_exp_level_1() {
        let exp = 0;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 1);
    }

    #[test]
    fn get_level_for_exp_negative_exp() {
        let exp = -10;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 1);
    }

    #[test]
    fn get_level_for_exp_level_4() {
        let exp = 300;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 4);
    }

    #[test]
    fn get_level_for_exp_level_5() {
        let exp = 400;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 5);
    }

    #[test]
    fn get_level_for_exp_level_8() {
        let exp = 840;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 8);
    }

    #[test]
    fn get_level_for_exp_level_20() {
        let exp = 4600;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 20);
    }

    #[test]
    fn get_level_for_exp_level_30() {
        let exp = 13800;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 30);
    }

    #[test]
    fn get_level_for_exp_level_40() {
        let exp = 39000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 40);
    }

    #[test]
    fn get_level_for_exp_level_50() {
        let exp = 105000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 50);
    }

    #[test]
    fn get_level_for_exp_level_60() {
        let exp = 290000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 60);
    }

    #[test]
    fn get_level_for_exp_level_70() {
        let exp = 750000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 70);
    }

    #[test]
    fn get_level_for_exp_level_80() {
        let exp = 2000000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 80);
    }

    #[test]
    fn get_level_for_exp_level_90() {
        let exp = 5500000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 90);
    }

    #[test]
    fn get_level_for_exp_level_99() {
        let exp = 14000000;
        let result = get_level_for_exp(exp);
        assert_eq!(result, 99);
    }
}
