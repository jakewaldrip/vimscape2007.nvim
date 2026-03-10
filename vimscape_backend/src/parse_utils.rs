use crate::{skills::Skills, token::Token};

pub fn parse_action_into_skill(token: &Token) -> Option<Skills> {
    match token {
        Token::MoveVerticalBasic(modifier) => Some(Skills::VerticalNavigation(*modifier)),
        Token::MoveHorizontalBasic(modifier) => Some(Skills::HorizontalNavigation(*modifier)),
        Token::MoveVerticalChunk(modifier) => Some(Skills::VerticalNavigation(modifier * 2)),
        Token::MoveHorizontalChunk(modifier) => Some(Skills::HorizontalNavigation(modifier * 2)),
        Token::JumpToHorizontal => Some(Skills::HorizontalNavigation(20)),
        Token::JumpToLineNumber(_) | Token::JumpToVertical => Some(Skills::VerticalNavigation(10)),
        Token::JumpFromContext | Token::Marks => Some(Skills::CodeFlow(25)),
        Token::CameraMovement => Some(Skills::CameraMovement(20)),
        Token::WindowManagement => Some(Skills::WindowManagement(25)),
        Token::TextManipulationBasic(modifier) | Token::DeleteText(modifier) => {
            Some(Skills::TextManipulation(*modifier))
        }
        Token::TextManipulationAdvanced => Some(Skills::TextManipulation(6)),
        Token::YankPaste => Some(Skills::Clipboard(6)),
        Token::UndoRedo => Some(Skills::Clipboard(5)),
        Token::DotRepeat => Some(Skills::Finesse(20)),
        Token::CommandSearch(completed) => Some(Skills::Search(if *completed { 15 } else { 1 })),
        Token::Command(completed) => Some(Skills::Finesse(if *completed { 15 } else { 2 })),
        Token::HelpPage(completed) => Some(Skills::Knowledge(if *completed { 50 } else { 1 })),
        Token::SaveFile(completed) => Some(Skills::Saving(if *completed { 20 } else { 1 })),
        Token::SearchRepeat => Some(Skills::Search(5)),
        Token::Unhandled(_) => None,
    }
}
