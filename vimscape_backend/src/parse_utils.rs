use crate::{skills::Skills, token::Token};

pub fn parse_action_into_skill(token: &Token) -> Option<Skills> {
    match token {
        Token::MoveVerticalBasic(modifier) => {
            let base_experience = 1;
            let experience = modifier * base_experience;
            Some(Skills::VerticalNavigation(experience))
        }
        Token::MoveHorizontalBasic(modifier) => {
            let base_experience = 1;
            let experience = modifier * base_experience;
            Some(Skills::HorizontalNavigation(experience))
        }
        Token::MoveVerticalChunk(modifier) => {
            let base_experience = 5;
            let experience = modifier * base_experience;
            Some(Skills::VerticalNavigation(experience))
        }
        Token::MoveHorizontalChunk(modifier) => {
            let base_experience = 5;
            let experience = modifier * base_experience;
            Some(Skills::HorizontalNavigation(experience))
        }
        Token::JumpToHorizontal => {
            let base_experience = 10;
            Some(Skills::HorizontalNavigation(base_experience))
        }
        Token::JumpToLineNumber(_line_number) => {
            let base_experience = 10;
            Some(Skills::VerticalNavigation(base_experience))
        }
        Token::JumpToVertical => {
            let base_experience = 10;
            Some(Skills::VerticalNavigation(base_experience))
        }
        Token::JumpFromContext => {
            let base_experience = 10;
            Some(Skills::CodeFlow(base_experience))
        }
        Token::CameraMovement => {
            let base_experience = 10;
            Some(Skills::CameraMovement(base_experience))
        }
        Token::WindowManagement => {
            let base_experience = 10;
            Some(Skills::WindowManagement(base_experience))
        }
        Token::TextManipulationBasic(modifier) | Token::DeleteText(modifier) => {
            let base_experience = 1;
            let experience = base_experience * modifier;
            Some(Skills::TextManipulation(experience))
        }
        Token::TextManipulationAdvanced => {
            let base_experience = 10;
            Some(Skills::TextManipulation(base_experience))
        }
        Token::YankPaste | Token::UndoRedo => {
            let base_experience = 10;
            Some(Skills::Clipboard(base_experience))
        }
        Token::DotRepeat => {
            let base_experience = 10;
            Some(Skills::Finesse(base_experience))
        }
        Token::CommandSearch(was_command_escaped) => {
            let base_experience = if *was_command_escaped { 1 } else { 10 };
            Some(Skills::Search(base_experience))
        }
        Token::Command(was_command_escaped) => {
            let base_experience = if *was_command_escaped { 1 } else { 10 };
            Some(Skills::Finesse(base_experience))
        }
        Token::HelpPage(was_command_escaped) => {
            let base_experience = if *was_command_escaped { 1 } else { 10 };
            Some(Skills::Knowledge(base_experience))
        }
        Token::SaveFile(was_command_escaped) => {
            let base_experience = if *was_command_escaped { 1 } else { 10 };
            Some(Skills::Saving(base_experience))
        }
        Token::Unhandled(token) => {
            println!("Unhandled token: {token}");
            None
        }
    }
}
