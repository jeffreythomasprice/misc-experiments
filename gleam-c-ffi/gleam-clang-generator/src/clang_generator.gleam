import argv
import clang
import gleam/list
import gleam/option

type Param {
  Param(name: String, typ: String)
}

type SimpleAST {
  FunctionDelcaration(
    file: String,
    name: String,
    return_type: String,
    params: List(Param),
  )
}

pub fn main() -> Nil {
  let argv.Argv(runtime: _, program:, arguments:) = argv.load()
  let source = case arguments {
    [source] -> source
    _ -> {
      panic as { "usage: " <> program <> " <source>" }
    }
  }

  let assert Ok(ast) = clang.parse_clang_ast_file(source)
  let assert Ok(ast) = parse_ast(ast)
  ast
  |> list.each(fn(ast) { echo ast as "TODO ast" })

  Nil
}

fn parse_ast(ast: clang.AST) -> Result(List(SimpleAST), String) {
  case ast {
    clang.AST(kind: "TranslationUnitDecl", inner:, ..) -> {
      Ok(
        inner
        |> list.filter_map(fn(ast) {
          case ast {
            clang.AST(
              kind: "FunctionDecl",
              loc: option.Some(clang.Loc(file: option.Some(file), ..)),
              name: option.Some(name),
              function_return_type: option.Some(return_type),
              inner:,
              ..,
            ) -> {
              let params =
                inner
                |> list.filter_map(fn(x) {
                  case x {
                    clang.AST(
                      kind: "ParmVarDecl",
                      name: option.Some(name),
                      typ: option.Some(clang.Type(qual_type: option.Some(
                        qual_type,
                      ))),
                      ..,
                    ) -> Ok(Param(name:, typ: qual_type))
                    _ -> Error("not a valid param")
                  }
                })
              Ok(FunctionDelcaration(file:, name:, return_type:, params:))
            }
            _ -> Error("not an AST type we care about")
          }
        }),
      )
    }
    clang.AST(kind:, ..) -> Error("wrong root AST type: " <> kind)
  }
}
