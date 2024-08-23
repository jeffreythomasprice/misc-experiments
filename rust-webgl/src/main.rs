mod demo;
mod dom;
mod events;
mod geometry;
mod graphics;

// use demo::camera_spinner_quad::DemoState;
use demo::ray_tracing::DemoState;
use log::*;
use std::panic;
use wasm_bindgen_futures::spawn_local;

fn main() {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    spawn_local(async {
        match events::run(Box::new(|context| match DemoState::new(context) {
            Ok(result) => Ok(Box::new(result)),
            Err(e) => Err(e),
        }))
        .await
        {
            Ok(_) => {
                warn!("state machine exited without error, but really it should keep going forever so something is probably wrong");
            }
            Err(e) => error!("state machine exited with: {e:?}"),
        };
    });
}
