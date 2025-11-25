#[derive(Debug)]
pub enum Skills {
    VerticalNavigation(i32),
    HorizontalNavigation(i32),
    CodeFlow(i32),
    CameraMovement(i32),
    WindowManagement(i32),
    TextManipulation(i32),
    Clipboard(i32),
    Finesse(i32),
    Search(i32),
    Knowledge(i32),
    Saving(i32),
}

impl Skills {
    pub fn to_str_vec() -> Vec<String> {
        let skills_strs: Vec<String> = [
            Skills::to_str(&Skills::VerticalNavigation(0)),
            Skills::to_str(&Skills::HorizontalNavigation(0)),
            Skills::to_str(&Skills::CodeFlow(0)),
            Skills::to_str(&Skills::CameraMovement(0)),
            Skills::to_str(&Skills::WindowManagement(0)),
            Skills::to_str(&Skills::TextManipulation(0)),
            Skills::to_str(&Skills::Clipboard(0)),
            Skills::to_str(&Skills::Finesse(0)),
            Skills::to_str(&Skills::Search(0)),
            Skills::to_str(&Skills::Knowledge(0)),
            Skills::to_str(&Skills::Saving(0)),
        ]
        .to_vec();
        skills_strs
    }

    pub fn to_str(&self) -> String {
        match self {
            Skills::VerticalNavigation(_) => "VerticalNavigation".to_string(),
            Skills::HorizontalNavigation(_) => "HorizontalNavigation".to_string(),
            Skills::CodeFlow(_) => "CodeFlow".to_string(),
            Skills::CameraMovement(_) => "CameraMovement".to_string(),
            Skills::WindowManagement(_) => "WindowManagement".to_string(),
            Skills::TextManipulation(_) => "TextManipulation".to_string(),
            Skills::Clipboard(_) => "Clipboard".to_string(),
            Skills::Finesse(_) => "Finesse".to_string(),
            Skills::Search(_) => "Search".to_string(),
            Skills::Knowledge(_) => "Knowledge".to_string(),
            Skills::Saving(_) => "Saving".to_string(),
        }
    }

    pub fn get_exp_from_skill(&self) -> i32 {
        match self {
            Skills::VerticalNavigation(exp)
            | Skills::HorizontalNavigation(exp)
            | Skills::CodeFlow(exp)
            | Skills::CameraMovement(exp)
            | Skills::WindowManagement(exp)
            | Skills::TextManipulation(exp)
            | Skills::Clipboard(exp)
            | Skills::Finesse(exp)
            | Skills::Search(exp)
            | Skills::Knowledge(exp)
            | Skills::Saving(exp) => *exp,
        }
    }
}
