use std::iter::repeat;

pub struct SkillData {
    pub skill_name: String,
    pub total_exp: i32,
    pub level: i32,
}

const MAX_NUM_COLS: i32 = 3;
// COL_WIDTH * MAX_NUM_COLS - MAX_NUM_COLS + 1 + MIN_SPACE
const MAX_WIDTH: i32 = 79;
// Width of 1 col, counts overlap
const COL_WIDTH: i32 = 25;
// Min buffer around the columns
const MIN_SPACE: i32 = 6;

pub fn format_skill_data(skill_data: &Vec<SkillData>, col_len: i32) -> Vec<String> {
    let num_cols = get_num_cols(&col_len);
    let mut lines: Vec<String> = Vec::new();

    // Error case, we can't even display one column
    if num_cols < 1 {
        lines.push("Screen size too small to display skills".into());
        return lines;
    }

    let horizontal_boundary_width: usize = (COL_WIDTH - 2).try_into().unwrap();
    let horizontal_boundary: String = repeat("─")
        .take(horizontal_boundary_width)
        .collect::<String>();

    let mut top_line: String = "┌".into();
    top_line.push_str(&horizontal_boundary);
    top_line.push('┐');

    let mut bottom_line: String = "└".into();
    bottom_line.push_str(&horizontal_boundary);
    bottom_line.push('┘');

    lines.push(top_line);

    // TODO: split this into columns, put separators between rows
    // put separators between columns
    for skill in skill_data {
        lines.push(skill.skill_name.clone());
        lines.push(skill.level.to_string());
        lines.push(skill.total_exp.to_string());
    }

    lines.push(bottom_line);

    lines
}

fn get_num_cols(col_len: &i32) -> i32 {
    // Cap num_cols at 3 wide
    if *col_len > MAX_WIDTH {
        return MAX_NUM_COLS;
    }

    let num_possible_cols = COL_WIDTH / (col_len - MIN_SPACE);
    if num_possible_cols > MAX_NUM_COLS {
        return MAX_NUM_COLS;
    }

    return num_possible_cols;
}
