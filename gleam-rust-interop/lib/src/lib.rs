use std::sync::Mutex;

use rustler::{Env, NifStruct, ResourceArc, Term};

// #[derive(NifStruct)]
// #[module = "RustLib.Data"]
struct Data {
    count: Mutex<i64>,
}

fn load(env: Env, _: Term) -> bool {
    println!("TODO load");
    rustler::resource!(Data, env);
    true
}

#[rustler::nif]
fn add_ints(a: i64, b: i64) -> i64 {
    a + b
}

#[rustler::nif]
fn new_data(initial_count: i64) -> ResourceArc<Data> {
    ResourceArc::new(Data {
        count: Mutex::new(initial_count),
    })
}

#[rustler::nif]
fn data_get(resource: ResourceArc<Data>) -> i64 {
    let count = resource.count.lock().unwrap();
    *count
}

#[rustler::nif]
fn data_increment(resource: ResourceArc<Data>, increment: i64) {
    let mut count = resource.count.lock().unwrap();
    *count += increment;
}

rustler::init!("rust_lib", load = load);
