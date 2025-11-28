use std::sync::Mutex;

use rustler::{Env, Resource, ResourceArc, Term};

struct Data {
    count: Mutex<i64>,
}

impl Resource for Data {}

fn load(env: Env, _: Term) -> bool {
    if !env.register::<Data>().is_ok() {
        return false;
    }
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
