use std::iter::repeat_n;

// Border chars
// │ ┌ ┐ └ ┘ ┬ ┴

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
// Total width for max columns: (columns * width) - (separators between columns) + 1 (for border) + min space
const MAX_WIDTH: i32 = (COL_WIDTH * MAX_NUM_COLS) - MAX_NUM_COLS + MIN_SPACE + 1;

pub fn format_skill_data(skill_data: &[SkillData], col_len: i32) -> Vec<String> {
    let num_cols = get_num_cols(col_len);
    let mut lines: Vec<String> = Vec::new();

    // Error case, we can't even display one column
    if num_cols < 1 {
        lines.push("Screen size too small to display skills".into());
        return lines;
    }

    // Padding
    let global_padding = get_global_left_padding(col_len, num_cols);

    let skill_batches: Vec<&[SkillData]> = skill_data.chunks(num_cols as usize).collect();

    for skill_batch in skill_batches {
        let top_line = create_boundary_line(
            i32::try_from(skill_batch.len()).unwrap(),
            &global_padding,
            '┌',
            '┐',
            '┬',
        );
        lines.push(top_line);

        let mut skill_line: String = global_padding.clone();
        let mut level_line: String = global_padding.clone();

        for skill in skill_batch {
            skill_line.push('│');
            level_line.push('│');

            // Center skill name: adjust left padding if odd length for better alignment
            let skill_char_count = i32::try_from(skill.skill_name.chars().count()).unwrap();
            let (skill_left_padding, skill_right_padding) =
                get_paddings(skill_char_count, skill_char_count % 2 != 0);

            // Format level with leading zero if < 10, and center with adjusted padding
            let level_str: String = if skill.level < 10 {
                format!("0{}", skill.level)
            } else {
                skill.level.to_string()
            };
            let level_char_count = i32::try_from(level_str.chars().count()).unwrap();
            let (level_left_padding, level_right_padding) =
                get_paddings(level_char_count, level_char_count % 2 != 0);

            skill_line.push_str(&skill_left_padding);
            skill_line.push_str(&skill.skill_name);
            skill_line.push_str(&skill_right_padding);

            level_line.push_str(&level_left_padding);
            level_line.push_str(&level_str);
            level_line.push_str(&level_right_padding);
        }

        skill_line.push('│');
        lines.push(skill_line);

        level_line.push('│');
        lines.push(level_line);

        let bottom_line = create_boundary_line(
            i32::try_from(skill_batch.len()).unwrap(),
            &global_padding,
            '└',
            '┘',
            '┴',
        );
        lines.push(bottom_line);
    }

    lines
}

fn create_boundary_line(
    num_cols: i32,
    global_padding: &str,
    left_corner: char,
    right_corner: char,
    junction: char,
) -> String {
    let segment_width: usize = (COL_WIDTH - 1)
        .try_into()
        .expect("Segment width calculation resulted in invalid usize");
    let segment: String = repeat_n("─", segment_width).collect::<String>();

    let mut line: String = global_padding.to_string();
    line.push(left_corner);
    for col in 0..num_cols {
        line.push_str(&segment);
        if col < num_cols - 1 {
            line.push(junction);
        }
    }
    line.push(right_corner);
    line
}

fn get_paddings(char_count: i32, adjust_left: bool) -> (String, String) {
    let base_padding = (COL_WIDTH - char_count) / 2;
    let left_amount = if adjust_left {
        (base_padding - 1).max(0)
    } else {
        base_padding.max(0)
    };
    let left_padding = repeat_n(" ", left_amount as usize).collect::<String>();
    let right_padding = repeat_n(" ", base_padding.max(0) as usize).collect::<String>();
    (left_padding, right_padding)
}

fn get_global_left_padding(col_len: i32, num_cols: i32) -> String {
    // Full width of the box: (columns * width) - columns (for separators) + 1 (for border)
    let full_box_width: i32 = (num_cols * COL_WIDTH) - num_cols + 1;
    let padding_amount: i32 = ((col_len - full_box_width) / 2).max(0);
    repeat_n(" ", padding_amount as usize).collect::<String>()
}

fn get_num_cols(col_len: i32) -> i32 {
    if col_len > MAX_WIDTH {
        return MAX_NUM_COLS;
    }

    let num_possible_cols = (col_len - MIN_SPACE) / COL_WIDTH;
    if num_possible_cols > MAX_NUM_COLS {
        return MAX_NUM_COLS;
    }

    num_possible_cols
}

pub fn format_skill_details(skill_data: &SkillData) -> Vec<String> {
    vec![
        format!("Experience - {}", skill_data.total_exp),
        format!("Level - {}", skill_data.level),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, level: i32) -> SkillData {
        SkillData {
            skill_name: name.to_string(),
            total_exp: 0,
            level,
        }
    }

    fn all_skills(level: i32) -> Vec<SkillData> {
        vec![
            make_skill("VerticalNavigation", level),
            make_skill("HorizontalNavigation", level),
            make_skill("CodeFlow", level),
            make_skill("CameraMovement", level),
            make_skill("WindowManagement", level),
            make_skill("TextManipulation", level),
            make_skill("Clipboard", level),
            make_skill("Finesse", level),
            make_skill("Search", level),
            make_skill("Knowledge", level),
            make_skill("Saving", level),
        ]
    }

    #[test]
    fn test_all_lines_have_consistent_width_level_below_10() {
        let skills = all_skills(1);
        let lines = format_skill_data(&skills, 100);

        // Lines are grouped in batches of 4 (top border, skill name, level, bottom border).
        // All lines within a batch must have the same character width.
        for batch in lines.chunks(4) {
            let widths: Vec<usize> = batch.iter().map(|l| l.chars().count()).collect();
            assert!(
                widths.windows(2).all(|w| w[0] == w[1]),
                "Line widths within a batch are inconsistent: {widths:?}\nLines:\n{}",
                batch.join("\n")
            );
        }
    }

    #[test]
    fn test_all_lines_have_consistent_width_level_above_10() {
        let skills = all_skills(42);
        let lines = format_skill_data(&skills, 100);

        for batch in lines.chunks(4) {
            let widths: Vec<usize> = batch.iter().map(|l| l.chars().count()).collect();
            assert!(
                widths.windows(2).all(|w| w[0] == w[1]),
                "Line widths within a batch are inconsistent: {widths:?}\nLines:\n{}",
                batch.join("\n")
            );
        }
    }

    #[test]
    fn test_all_lines_have_consistent_width_mixed_levels() {
        let skills = vec![
            make_skill("VerticalNavigation", 5),
            make_skill("HorizontalNavigation", 15),
            make_skill("CodeFlow", 99),
            make_skill("CameraMovement", 1),
            make_skill("WindowManagement", 10),
            make_skill("TextManipulation", 3),
        ];
        let lines = format_skill_data(&skills, 100);

        for batch in lines.chunks(4) {
            let widths: Vec<usize> = batch.iter().map(|l| l.chars().count()).collect();
            assert!(
                widths.windows(2).all(|w| w[0] == w[1]),
                "Line widths within a batch are inconsistent: {widths:?}\nLines:\n{}",
                batch.join("\n")
            );
        }
    }
}
