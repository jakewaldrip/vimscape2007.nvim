use std::iter::repeat;

// Border chars
// │ ┌ ┐ └ ┘

pub struct SkillData {
    pub skill_name: String,
    pub total_exp: i32,
    pub level: i32,
}

const MAX_NUM_COLS: i32 = 3;
// Width of 1 col, counts overlap
const COL_WIDTH: i32 = 25;
// Min buffer around the columns
const MIN_SPACE: i32 = 6;
// (COL_WIDTH * MAX_NUM_COLS) - MAX_NUM_COLS + 1 + MIN_SPACE
const MAX_WIDTH: i32 = (COL_WIDTH * MAX_NUM_COLS) - MAX_NUM_COLS + MIN_SPACE + 1;

pub fn format_skill_data(skill_data: &Vec<SkillData>, col_len: i32) -> Vec<String> {
    let num_cols = get_num_cols(&col_len);
    let mut lines: Vec<String> = Vec::new();

    // Error case, we can't even display one column
    if num_cols < 1 {
        lines.push("Screen size too small to display skills".into());
        return lines;
    }

    // Padding
    let global_padding = get_global_left_padding(&col_len, &num_cols);

    let batched_skills: Vec<&[SkillData]> = skill_data.chunks(num_cols as usize).collect();

    for skill_batch in batched_skills {
        let top_line = create_top_line(&(skill_batch.len() as i32), global_padding.clone());
        lines.push(top_line.clone());

        let mut curr_skill_line: String = global_padding.clone();
        let mut curr_level_line: String = global_padding.clone();

        for skill in skill_batch {
            curr_skill_line.push('│');
            curr_level_line.push('│');

            let skill_char_count = skill.skill_name.chars().count() as i32;
            let level_char_count = skill.level.to_string().chars().count() as i32;

            let skill_padding = get_padding(&skill_char_count);
            let skill_adjusted_padding = get_adjusted_padding(&skill_char_count);
            let level_padding = get_padding(&level_char_count);

            let skill_left_padding = if skill_char_count % 2 == 0 {
                &skill_padding
            } else {
                &skill_adjusted_padding
            };

            let level_str: String = if skill.level < 10 {
                format!("0{}", skill.level.to_string())
            } else {
                skill.level.to_string()
            };

            let level_left_padding: String = if skill.level < 10 {
                get_adjusted_padding(&level_char_count)
            } else {
                level_padding.clone()
            };
            let level_right_padding: String = if skill.level < 10 {
                get_adjusted_padding(&level_char_count)
            } else {
                level_padding.clone()
            };

            curr_skill_line.push_str(&skill_left_padding);
            curr_skill_line.push_str(&skill.skill_name);
            curr_skill_line.push_str(&skill_padding);

            curr_level_line.push_str(&level_left_padding);
            curr_level_line.push_str(&level_str);
            curr_level_line.push_str(&level_right_padding);
        }

        curr_skill_line.push('│');
        lines.push(curr_skill_line);

        curr_level_line.push('│');
        lines.push(curr_level_line);

        let bottom_line = create_bottom_line(&(skill_batch.len() as i32), global_padding.clone());
        lines.push(bottom_line.clone());
    }

    // ----
    lines
}

fn create_top_line(num_cols: &i32, global_padding: String) -> String {
    let horizontal_boundary_width: usize = ((COL_WIDTH * num_cols) - 1).try_into().unwrap();
    let horizontal_line: String = repeat("─")
        .take(horizontal_boundary_width)
        .collect::<String>();

    let mut top_line: String = global_padding;
    top_line.push('┌');
    top_line.push_str(&horizontal_line);
    top_line.push('┐');
    return top_line.clone();
}

fn create_bottom_line(num_cols: &i32, global_padding: String) -> String {
    let horizontal_boundary_width: usize = ((COL_WIDTH * num_cols) - 1).try_into().unwrap();
    let horizontal_line: String = repeat("─")
        .take(horizontal_boundary_width)
        .collect::<String>();

    let mut bottom_line: String = global_padding;
    bottom_line.push('└');
    bottom_line.push_str(&horizontal_line);
    bottom_line.push('┘');

    return bottom_line.clone();
}

fn get_padding(char_count: &i32) -> String {
    let padding_amount: i32 = (COL_WIDTH - char_count) / 2;
    let padding_space: String = repeat(" ")
        .take(padding_amount as usize)
        .collect::<String>();
    padding_space
}

fn get_adjusted_padding(char_count: &i32) -> String {
    let padding_amount: i32 = (COL_WIDTH - char_count) / 2;
    let adjusted_padding_space: String = repeat(" ")
        .take((padding_amount - 1) as usize)
        .collect::<String>();
    adjusted_padding_space
}

fn get_global_left_padding(col_len: &i32, num_cols: &i32) -> String {
    let full_box_width: i32 = (*num_cols * COL_WIDTH) - MAX_NUM_COLS + 1;
    let padding_amount: i32 = (*col_len - full_box_width) / 2;
    let padding_space: String = repeat(" ")
        .take(padding_amount as usize)
        .collect::<String>();
    padding_space
}

fn get_num_cols(col_len: &i32) -> i32 {
    if *col_len > MAX_WIDTH {
        return MAX_NUM_COLS;
    }

    let num_possible_cols = (*col_len - MIN_SPACE) / COL_WIDTH;
    if num_possible_cols > MAX_NUM_COLS {
        return MAX_NUM_COLS;
    }

    return num_possible_cols;
}

pub fn format_skill_details(skill_data: &SkillData) -> Vec<String> {
    let SkillData {
        total_exp,
        level: _,
        skill_name,
    } = skill_data;

    let mut lines: Vec<String> = Vec::new();
    lines.push(skill_name.clone());
    lines.push(total_exp.to_string().clone());
    lines
}
