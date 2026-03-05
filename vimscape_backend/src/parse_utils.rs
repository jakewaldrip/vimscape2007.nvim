use crate::{skills::Skills, token::Token};

pub fn parse_action_into_skill(token: &Token) -> Option<Skills> {
    match token {
        Token::MoveVerticalBasic(modifier) => Some(Skills::VerticalNavigation(*modifier)),
        Token::MoveHorizontalBasic(modifier) => Some(Skills::HorizontalNavigation(*modifier)),
        Token::MoveVerticalChunk(modifier) => Some(Skills::VerticalNavigation(modifier * 5)),
        Token::MoveHorizontalChunk(modifier) => Some(Skills::HorizontalNavigation(modifier * 5)),
        Token::JumpToHorizontal => Some(Skills::HorizontalNavigation(10)),
        Token::JumpToLineNumber(_) | Token::JumpToVertical => Some(Skills::VerticalNavigation(10)),
        Token::JumpFromContext | Token::Marks => Some(Skills::CodeFlow(10)),
        Token::CameraMovement => Some(Skills::CameraMovement(10)),
        Token::WindowManagement => Some(Skills::WindowManagement(10)),
        Token::TextManipulationBasic(modifier) | Token::DeleteText(modifier) => {
            Some(Skills::TextManipulation(*modifier))
        }
        Token::TextManipulationAdvanced => Some(Skills::TextManipulation(10)),
        Token::YankPaste | Token::UndoRedo => Some(Skills::Clipboard(10)),
        Token::DotRepeat => Some(Skills::Finesse(10)),
        Token::CommandSearch(completed) => Some(Skills::Search(if *completed { 1 } else { 10 })),
        Token::Command(completed) => Some(Skills::Finesse(if *completed { 1 } else { 10 })),
        Token::HelpPage(completed) => Some(Skills::Knowledge(if *completed { 1 } else { 10 })),
        Token::SaveFile(completed) => Some(Skills::Saving(if *completed { 1 } else { 10 })),
        Token::SearchRepeat => Some(Skills::Search(5)),
        Token::Unhandled(_) => None,
    }
}
