mod dom;
mod events;
mod geom;

use anyhow::Result;
use events::EventHandler;
use log::*;
use std::panic;
use web_sys::WebGl2RenderingContext;

struct DemoState {}

impl EventHandler for DemoState {
    fn activate(&mut self, context: &WebGl2RenderingContext) -> Result<()> {
        context.clear_color(0.25, 0.5, 1.0, 1.0);

        Ok(())
    }

    fn deactivate(&mut self) -> Result<()> {
        Ok(())
    }

    fn resize(
        &mut self,
        context: &WebGl2RenderingContext,
        size: geom::Size<u32>,
    ) -> Result<events::NextEventHandler> {
        context.viewport(0, 0, size.width as i32, size.height as i32);

        Ok(events::NextEventHandler::NoChange)
    }

    fn render(&mut self, context: &WebGl2RenderingContext) -> Result<events::NextEventHandler> {
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        Ok(events::NextEventHandler::NoChange)
    }

    fn update(&mut self, delta: chrono::TimeDelta) -> Result<events::NextEventHandler> {
        Ok(events::NextEventHandler::NoChange)
    }
}

fn main() -> Result<()> {
    console_log::init_with_level(Level::Trace).unwrap();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    events::run(Box::new(DemoState {}))
}
