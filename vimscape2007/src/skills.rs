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
            "VerticalNavigation".to_string(),
            "HorizontalNavigation".to_string(),
            "CodeFlow".to_string(),
            "CameraMovement".to_string(),
            "WindowManagement".to_string(),
            "TextManipulation".to_string(),
            "Clipboard".to_string(),
            "Finesse".to_string(),
            "Search".to_string(),
            "Knowledge".to_string(),
            "Saving".to_string(),
        ]
        .to_vec();
        skills_strs
    }
}
