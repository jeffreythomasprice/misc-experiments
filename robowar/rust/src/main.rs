use std::error::Error;

use crate::assembler::assemble;

mod assembler;
mod emulator;

fn main() -> Result<(), Box<dyn Error>> {
    assemble(r"foo");
    Ok(())
}
