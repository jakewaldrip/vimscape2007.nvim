use crate::{skills::Skills, token::Token};

pub fn parse_action_into_skill(token: Result<Token, ()>) -> Option<Skills> {
    match token {
        Ok(Token::MoveVerticalBasic(modifier)) => {
            let base_experience = 1;
            let experience = modifier * base_experience;
            Some(Skills::VerticalNavigation(experience))
        }
        Ok(Token::MoveHorizontalBasic(modifier)) => {
            let base_experience = 1;
            let experience = modifier * base_experience;
            Some(Skills::HorizontalNavigation(experience))
        }
        Ok(Token::MoveVerticalChunk) => {
            let base_experience = 10;
            Some(Skills::VerticalNavigation(base_experience))
        }
        Ok(Token::MoveHorizontalChunk(modifier)) => {
            let base_experience = 5;
            let experience = modifier * base_experience;
            Some(Skills::HorizontalNavigation(experience))
        }
        Ok(Token::JumpToHorizontal) => {
            let base_experience = 10;
            Some(Skills::HorizontalNavigation(base_experience))
        }
        Ok(Token::JumpToLineNumber(_line_number)) => {
            let base_experience = 10;
            Some(Skills::VerticalNavigation(base_experience))
        }
        Ok(Token::JumpToVertical) => {
            let base_experience = 10;
            Some(Skills::VerticalNavigation(base_experience))
        }
        Ok(Token::JumpFromContext) => {
            let base_experience = 10;
            Some(Skills::CodeFlow(base_experience))
        }
        Ok(Token::CameraMovement) => {
            let base_experience = 10;
            Some(Skills::CameraMovement(base_experience))
        }
        Ok(Token::WindowManagement) => {
            let base_experience = 10;
            Some(Skills::WindowManagement(base_experience))
        }
        Ok(Token::TextManipulationBasic(modifier)) => {
            let base_experience = 1;
            let experience = base_experience * modifier;
            Some(Skills::TextManipulation(experience))
        }
        Ok(Token::TextManipulationAdvanced) => {
            let base_experience = 10;
            Some(Skills::TextManipulation(base_experience))
        }
        Ok(Token::YankPaste) => {
            let base_experience = 10;
            Some(Skills::Clipboard(base_experience))
        }
        Ok(Token::UndoRedo) => {
            let base_experience = 10;
            Some(Skills::Clipboard(base_experience))
        }
        Ok(Token::DotRepeat) => {
            let base_experience = 10;
            Some(Skills::Finesse(base_experience))
        }
        Ok(Token::CommandSearch(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            Some(Skills::Search(base_experience))
        }
        Ok(Token::DeleteText(modifier)) => {
            let base_experience = 1;
            let experience = base_experience * modifier;
            Some(Skills::TextManipulation(experience))
        }
        Ok(Token::Command(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            Some(Skills::Finesse(base_experience))
        }
        Ok(Token::HelpPage(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            Some(Skills::Knowledge(base_experience))
        }
        Ok(Token::SaveFile(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            Some(Skills::Saving(base_experience))
        }
        _ => None,
    }
}
