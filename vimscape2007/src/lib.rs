use motions::Motions;
use nvim_oxi::{self as oxi, print, Dictionary, Function};

mod motions;

#[nvim_oxi::plugin]
fn vimscape2007() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let api = Dictionary::from_iter([("process_batch", process_batch_fn)]);
    Ok(api)
}

fn process_batch(input: String) -> bool {
    print!("Processing Batch via Rust, input: {} ", input);
    let mut motions = input_string_to_motions_vec(input);
    true
}

fn input_string_to_motions_vec(input: String) -> Vec<Motions> {
    let mut motions = Vec::<Motions>::new();
    motions
}

#[oxi::test]
fn process_batch_succeeds_base_case() {
    let result = process_batch("".to_string());
    assert_eq!(result, true);
}

#[oxi::test]
fn motions_transform_succeeds_base_case() {
    let result = input_string_to_motions_vec("".to_string());
    let expected: Vec<Motions> = vec![];
    assert_eq!(result, expected);
}
