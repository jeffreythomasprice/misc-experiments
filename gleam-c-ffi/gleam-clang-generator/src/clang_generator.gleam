import argv
import clang

pub fn main() -> Nil {
  let argv.Argv(runtime: _, program:, arguments:) = argv.load()
  let source = case arguments {
    [source] -> source
    _ -> {
      panic as { "usage: " <> program <> " <source>" }
    }
  }

  let assert Ok(ast) = clang.parse_clang_ast_file(source)
  echo ast as "TODO ast"

  Nil
}
