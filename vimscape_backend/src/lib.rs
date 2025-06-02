use api::{get_skill_details, get_user_data, process_batch, setup_tables};
use nvim_oxi::{Dictionary, Function, Object};

mod api;
mod db;
mod levels;
mod parse_utils;
mod parser;
mod skill_data;
mod skills;
mod token;

#[nvim_oxi::plugin]
fn vimscape_backend() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let get_user_data_fn = Function::from_fn(get_user_data);
    let setup_tables_fn = Function::from_fn(setup_tables);
    let get_skill_details_fn = Function::from_fn(get_skill_details);
    let api = Dictionary::from_iter([
        ("process_batch", Object::from(process_batch_fn)),
        ("get_user_data", Object::from(get_user_data_fn)),
        ("setup_tables", Object::from(setup_tables_fn)),
        ("get_skill_details", Object::from(get_skill_details_fn)),
    ]);
    Ok(api)
}
