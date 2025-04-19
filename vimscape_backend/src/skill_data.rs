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

    let mut batched_skills: Vec<&[SkillData]> = skill_data.chunks(num_cols as usize).collect();

    // Temp testing for single skill columns
    let skill_data_test = SkillData {
        skill_name: "Bobby".into(),
        total_exp: 32,
        level: 15,
    };
    let test = vec![skill_data_test];
    let test2: &[SkillData] = &test;
    batched_skills.push(test2);

    for skill_batch in batched_skills {
        let top_line = create_top_line(&(skill_batch.len() as i32));
        lines.push(top_line.clone());
        let mut curr_skill_line: String = "".into();
        for skill in skill_batch {
            curr_skill_line.push('│');

            let char_count = skill.skill_name.chars().count() as i32;
            let padding_space = get_padding_space(&char_count);
            let adjusted_padding_space = get_adjusted_padding_space(&char_count);
            let left_padding = if char_count % 2 == 0 {
                &padding_space
            } else {
                &adjusted_padding_space
            };

            curr_skill_line.push_str(&left_padding);
            curr_skill_line.push_str(&skill.skill_name);
            curr_skill_line.push_str(&padding_space);
        }

        curr_skill_line.push('│');
        lines.push(curr_skill_line);

        let bottom_line = create_bottom_line(&(skill_batch.len() as i32));
        lines.push(bottom_line.clone());
    }

    // ----
    lines
}

fn create_top_line(num_cols: &i32) -> String {
    let horizontal_boundary_width: usize = ((COL_WIDTH * num_cols) - 1).try_into().unwrap();
    let horizontal_line: String = repeat("─")
        .take(horizontal_boundary_width)
        .collect::<String>();

    let mut top_line: String = "┌".into();
    top_line.push_str(&horizontal_line);
    top_line.push('┐');
    return top_line.clone();
}

fn create_bottom_line(num_cols: &i32) -> String {
    let horizontal_boundary_width: usize = ((COL_WIDTH * num_cols) - 1).try_into().unwrap();
    let horizontal_line: String = repeat("─")
        .take(horizontal_boundary_width)
        .collect::<String>();

    let mut bottom_line: String = "└".into();
    bottom_line.push_str(&horizontal_line);
    bottom_line.push('┘');

    return bottom_line.clone();
}

fn get_padding_space(char_count: &i32) -> String {
    let padding_amount: i32 = (COL_WIDTH - char_count) / 2;
    let padding_space: String = repeat(" ")
        .take(padding_amount as usize)
        .collect::<String>();
    padding_space
}

fn get_adjusted_padding_space(char_count: &i32) -> String {
    let padding_amount: i32 = (COL_WIDTH - char_count) / 2;
    let adjusted_padding_space: String = repeat(" ")
        .take((padding_amount - 1) as usize)
        .collect::<String>();
    adjusted_padding_space
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
