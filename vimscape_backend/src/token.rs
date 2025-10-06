// TODO: Check what a <leader>X renders as
#[derive(Debug, Clone)]
pub enum Token {
    // gj, gk, j, k, 10j,
    MoveVerticalBasic(i32),

    Unhandled(String),

    // 10h, h, l
    MoveHorizontalBasic(i32),
    //
    // // 10<C-U>, <C-U>, <C-D>
    // MoveVerticalChunk(String),
    //
    // // 10w, w, W, e, E, b, B
    // MoveHorizontalChunk(String),
    //
    // // _g_, F, f, T, t, + any 1 char
    // JumpToHorizontal(String),
    //
    // // :10|enter|, 10gg, gg, G
    // JumpToLineNumber(String),
    //
    // // M, H, L, <C-F>, <C-B>
    // JumpToVertical(String),
    //
    // // % renders as the token below
    // // :<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zv
    // JumpFromContext(String),
    //
    // // zz renders as zzz
    // // zb, zt, zzz, <C-E>, <C-Y>
    // CameraMovement(String),
    //
    // // <C-W>(svwqx=hljkHLJK), <C-H>, <C-J>, <C-K>, <C-L>
    // WindowManagement(String),
    //
    // // x renders as xdl
    // // [num](xdl, gJ, J, r[character])
    // TextManipulationBasic(String),
    //
    // // R[character]<Esc>, g(~uU)[num](wWeEbB$^0fFtT),
    // // ci))<C-\><C-N>zvzvv
    // // ci((<C-\><C-N>zvzvv
    // // ci[[<C-\><C-N>zvzvv
    // // ci]]<C-\><C-N>zvzvv
    // // ci{{<C-\><C-N>zvzvv
    // // ci}}<C-\><C-N>zvzvv
    // // c$$, cC$, cee, cww, scl, Scc, ciwwiw, cawwaw
    // TextManipulationAdvanced(String),
    //
    // // p renders as ""1p
    // // yy renders as yyy
    // // Y renders as y$
    // // y<Esc> renders as y<Esc><C-\><C-N><Esc>, i can't explain that one
    // // [num]""(p, P), [num]y($, w, iw, aw, yy), [num]y<Esc><C-\><C-N><Esc>
    // YankPaste(String),
    //
    // // u, U, <C-R>
    // UndoRedo(String),
    //
    // // This literally just repeats the keys
    // // so i will just let it grant xp for both categories
    // // .
    // DotRepeat(String),
    //
    // // /[any characters]
    // CommandSearch(String),
    //
    // // d$, dw, etc, doubles up on every key, ie dww, d$$, etc
    // // If a number is included, it doubles the digits of the number
    // // [num doubled last digit](dww, d$$, dWW, dee, dEE, dbb, dBB, d$$, d^^, d00)
    // DeleteText(String),
    //
    // // :[any characters]
    // Command(String),
    //
    // // :(h, help)[any characters](|enter|, <Esc>)
    // HelpPage(String),
    //
    // // :w(|enter|, <Esc>)
    // SaveFile(String),
}
