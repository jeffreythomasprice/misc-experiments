#[rustler::nif]
pub fn foo() -> i64 {
    return 42;
}

rustler::init!("rust_lib", [foo]);
