enum HorizontalModifiers {
    Line,
    Word,
}

enum VerticalModifiers {
    Exact(i32),
    QuickScroll,
}

enum Motions {
    MoveDown(VerticalModifiers),
    MoveUp(VerticalModifiers),
    MoveLeft(HorizontalModifiers),
    MoveRight(HorizontalModifiers),
}
