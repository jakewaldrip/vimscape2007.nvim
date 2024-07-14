use nvim_oxi::{print, Dictionary, Function};

#[nvim_oxi::plugin]
fn vimscape2007() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let api = Dictionary::from_iter([("process_batch", process_batch_fn)]);
    Ok(api)
}

fn process_batch(input: String) -> bool {
    print!("Processing Batch via Rust, {} characters", input.len());
    true
}
