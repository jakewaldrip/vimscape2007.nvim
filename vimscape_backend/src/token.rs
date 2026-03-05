#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // gj, gk, j, k, 10j,
    MoveVerticalBasic(i32),

    Unhandled(String),

    // 10h, h, l
    MoveHorizontalBasic(i32),

    // 10<C-U>, <C-U>, <C-D>
    MoveVerticalChunk(i32),

    // 10w, w, W, e, E, b, B
    MoveHorizontalChunk(i32),

    // f, F, t, T + any 1 char
    JumpToHorizontal,

    // :10|enter|, 10gg, gg, G
    #[allow(dead_code)]
    JumpToLineNumber(String),

    // M, H, L, <C-F>, <C-B>
    JumpToVertical,

    // %
    JumpFromContext,

    // zz, zb, zt, <C-E>, <C-Y>
    CameraMovement,

    // <C-W>(svwqx=hljkHLJK), <C-H>, <C-J>, <C-K>, <C-L>
    WindowManagement,

    // [num](x, X, gJ, J, r[character])
    TextManipulationBasic(i32),

    // R[character]|escape|, g(~uU)[num](wWeEbB$^0fFtT),
    // cc, cw, c$, ciw, caw, ci), ca}, s, S, C, ~
    TextManipulationAdvanced,

    // p, P, Y, yy, yw, y$, yiw, yaw
    YankPaste,

    // u, U, <C-R>
    UndoRedo,

    // .
    DotRepeat,

    // /[any characters] followed by |escape| or |enter|
    CommandSearch(bool),

    // n, N (repeat last search), ;, , (repeat last f/F/t/T)
    SearchRepeat,

    // m{char} (set mark), '{char} (jump to mark line), `{char} (jump to mark position)
    Marks,

    // dd, dw, d$, dW, de, dE, db, dB, d^, d0, D, diw, daw, di), da}
    DeleteText(i32),

    // :[any characters] followed by |escape| or |enter|
    Command(bool),

    // :(h, help)[any characters] followed by |enter| or |escape|
    HelpPage(bool),

    // :w followed by |enter| or |escape|
    SaveFile(bool),
}
