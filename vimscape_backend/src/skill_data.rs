use std::iter::repeat;

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
const MAX_WIDTH: i32 = (COL_WIDTH * 3) - MAX_NUM_COLS + MIN_SPACE + 1;

pub fn format_skill_data(skill_data: &Vec<SkillData>, col_len: i32) -> Vec<String> {
    let num_cols = get_num_cols(&col_len);
    let mut lines: Vec<String> = Vec::new();

    // Error case, we can't even display one column
    if num_cols < 1 {
        lines.push("Screen size too small to display skills".into());
        return lines;
    }

    // Get the full width of the columns together
    let global_horizontal_boundary_width: usize =
        (((COL_WIDTH - 1) * num_cols) + 2).try_into().unwrap();
    let horizontal_cell_boundary: String = repeat("─")
        .take(global_horizontal_boundary_width)
        .collect::<String>();

    // Padding
    lines.push("".into());

    // │
    //
    // ┌
    //
    // ┐
    //
    // └
    //
    // ┘

    let mut global_top_line: String = "┌".into();
    global_top_line.push_str(&horizontal_cell_boundary);
    global_top_line.push('┐');

    let mut global_bottom_line: String = "└".into();
    global_bottom_line.push_str(&horizontal_cell_boundary);
    global_bottom_line.push('┘');

    let batched_skills: Vec<&[SkillData]> = skill_data.chunks(num_cols as usize).collect();
    for skill_batch in batched_skills {
        lines.push(global_top_line.clone());
        let mut curr_skill_line: String = "".into();
        for skill in skill_batch {
            let char_count = skill.skill_name.chars().count() as i32;
            let padding_amount: i32 = (COL_WIDTH - char_count) / 2;
            let padding_space: String = repeat(" ")
                .take(padding_amount as usize)
                .collect::<String>();
            let adjusted_padding_space: String = repeat(" ")
                .take((padding_amount - 1) as usize)
                .collect::<String>();

            curr_skill_line.push('│');

            // handle odd number skill name spacing issue
            if char_count % 2 == 0 {
                curr_skill_line.push_str(&padding_space);
            } else {
                curr_skill_line.push_str(&adjusted_padding_space);
            }

            curr_skill_line.push_str(&skill.skill_name);
            curr_skill_line.push_str(&padding_space);
        }

        curr_skill_line.push('│');
        lines.push(curr_skill_line);
        lines.push(global_bottom_line.clone());
    }

    // ----
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
