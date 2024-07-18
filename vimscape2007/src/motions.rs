#[derive(Debug, PartialEq)]
pub enum HorizontalModifiers {
    Line,
    Word,
}

#[derive(Debug, PartialEq)]
pub enum VerticalModifiers {
    Exact(i32),
    QuickScroll,
}

#[derive(Debug, PartialEq)]
pub enum Motions {
    MoveDown(VerticalModifiers),
    MoveUp(VerticalModifiers),
    MoveLeft(HorizontalModifiers),
    MoveRight(HorizontalModifiers),
}
