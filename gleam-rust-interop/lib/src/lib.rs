#[rustler::nif]
pub fn add_ints(a: i64, b: i64) -> i64 {
    a + b
}

rustler::init!("rust_lib", [add_ints]);
