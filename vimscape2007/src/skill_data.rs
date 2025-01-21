pub struct SkillData {
    pub skill_name: String,
    pub total_exp: i32,
    pub level: i32,
}

const MAX_NUM_COLS: i32 = 3;
// max col_len we consider
const MAX_WIDTH: i32 = 42;
// width of 1 col, counts overlap (this * num_cols - num_cols + 1 = allocated width)
const COL_WIDTH: i32 = 20;

pub fn format_skill_data(_skill_data: &Vec<SkillData>, col_len: i32) -> Vec<String> {
    let num_cols = get_num_cols(&col_len);
    let mut lines: Vec<String> = Vec::new();
    lines.push("Test Line".into());
    lines.push(col_len.to_string());
    lines.push(num_cols.to_string());
    lines
}

fn get_num_cols(col_len: &i32) -> i32 {
    // Cap num_cols at 3 wide
    if *col_len > MAX_WIDTH {
        return MAX_NUM_COLS;
    }

    // TODO calculate number of cols allowed based on width
    // Roughly, col_len / col_width, take whole number as col width
    // Check decimal to ensure some spacing (at least like .1 or something)
    //
    //
    13
}
