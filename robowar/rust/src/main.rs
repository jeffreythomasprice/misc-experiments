mod math;
mod parser;
mod simulation;
mod window;


use color_eyre::eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::window::{EventHandler, run};

struct Demo {}

impl EventHandler for Demo {
    fn render(&mut self, buffer: &mut [u32], width: u32, height: u32) -> Result<()> {
        for index in 0..(width * height) {
            let x = index % width;
            let y = index / width;
            let r = x % 255;
            let g = y % 255;
            let b = 255 - (x % 255);
            buffer[index as usize] = b | (g << 8) | (r << 16);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    run(Demo {})
}
