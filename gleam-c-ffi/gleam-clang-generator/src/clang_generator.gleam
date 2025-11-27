import gleam/int
import gleam/io
import gleamyshell

pub fn main() -> Nil {
  case
    gleamyshell.execute("clang", in: "..", args: [
      "-Xclang",
      "-ast-dump=json",
      "c-lib/src/lib.h",
    ])
  {
    Ok(gleamyshell.CommandOutput(exit_code: 0, output:)) -> {
      io.println("TODO output: " <> output)
    }
    Ok(gleamyshell.CommandOutput(exit_code:, output:)) -> {
      panic as {
        "error, exit code: "
        <> int.to_string(exit_code)
        <> ", output:\n"
        <> output
      }
    }
    Error(e) -> panic as { "error running command: " <> e }
  }

  Nil
}
