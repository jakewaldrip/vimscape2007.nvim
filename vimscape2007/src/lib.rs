use nvim_oxi::{print, Dictionary, Function};

mod motions;
mod utils;

#[nvim_oxi::plugin]
fn vimscape2007() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let api = Dictionary::from_iter([("process_batch", process_batch_fn)]);
    Ok(api)
}

fn process_batch(input: String) -> bool {
    print!("Processing Batch via Rust, input: {} ", input);
    true

    // Basic motion enums are defined
    // Need to read over the string
    // maybe define some helpers or find them? (ie peek, peek_to_next_motion?)
    //
    // I need a top level vector to store these motion enums inside of
    // log it into a file for testing (in utils)
    //
    // Look into techniques for parsing strings as essentially a language
    // Parsers would be good reading material
}
