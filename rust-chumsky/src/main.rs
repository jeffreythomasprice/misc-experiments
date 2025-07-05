use crate::parser::program;
use chumsky::prelude::*;

mod emulator;
mod instruction_set;
mod parser;

fn main() {
    let input = r"
        main:
            add r1, r2, 42
        foo:
        bar:
            add r34, r34, -1
    "
    .to_string();

    let parser = program();

    match parser.parse(&input).into_result() {
        Ok(result) => {
            // for statement in result.iter() {
            //     println!("statement: {:?}", statement);
            // }
            println!("program: {:?}", result);
        }
        Result::Err(e) => println!("failed: {:?}", e),
    };
}
