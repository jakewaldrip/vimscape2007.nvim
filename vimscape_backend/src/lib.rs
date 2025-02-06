use api::{get_user_data, process_batch, setup_tables};
use nvim_oxi::{self as oxi, Dictionary, Function, Object};

mod api;
mod db;
mod parse_utils;
mod skill_data;
mod skills;
mod token;

#[nvim_oxi::plugin]
fn vimscape_backend() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let get_user_data_fn = Function::from_fn(get_user_data);
    let setup_tables_fn = Function::from_fn(setup_tables);
    let api = Dictionary::from_iter([
        ("process_batch", Object::from(process_batch_fn)),
        ("get_user_data", Object::from(get_user_data_fn)),
        ("setup_tables", Object::from(setup_tables_fn)),
    ]);
    Ok(api)
}

#[oxi::test]
fn process_batch_succeeds_base_case() {
    let result = process_batch(("".to_string(), "".into()));
    assert_eq!(result, true);
}

#[oxi::test]
fn process_batch_prints_tokens_test() {
    setup_tables("".into());
    let result = process_batch(
        (r#"jk3l:w|enter|hd33ww:<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zvzzz:h test<Esc>jj:help test|enter|<C-W>s<C-W>v3""3puU<C-R>3w.3w/testsearch|enter|/testsearch2<Esc>hjkl"#
            .to_string(),
        "".into())
    );
    assert_eq!(result, true);
}

#[oxi::test]
fn get_user_data_base_case() {
    setup_tables("".into());
    let result = get_user_data((30, "".into()));
    println!("result {:?}", result);
    assert_eq!(1, 1);
}
