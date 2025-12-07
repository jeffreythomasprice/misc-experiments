import argv
import clang
import gleam/io
import gleam/list
import gleam/option
import gleam/set
import gleam/string

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

  let assert Ok(ast) = new_simple_ast_from_path(source)
  ast
  |> list.each(fn(ast) { io.println("TODO ast:\n" <> string.inspect(ast)) })
  io.println("")

  let assert Ok(nif_c_source) =
    simple_ast_create_nif_c_source(ast:, header_path: source)
  io.println("TODO nif c:\n" <> nif_c_source <> "\n")

  // TODO the .erl file

  Nil
}

fn new_simple_ast_from_clang_ast(
  ast: clang.AST,
) -> Result(List(SimpleAST), String) {
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

fn new_simple_ast_from_path(path: String) -> Result(List(SimpleAST), String) {
  let assert Ok(ast) = clang.parse_clang_ast_file(path)
  new_simple_ast_from_clang_ast(ast)
}

fn simple_ast_create_nif_c_source(
  ast ast: List(SimpleAST),
  header_path header_path: String,
) -> Result(String, String) {
  let parts = ["#include <erl_nif.h>", "#include \"" <> header_path <> "\""]

  let parts =
    list.append(
      parts,
      ast
        |> list.map(fn(ast) {
          case ast {
            FunctionDelcaration(file: _, name:, return_type:, params:) -> {
              let parts = [
                "static ERL_NIF_TERM "
                <> name
                <> "_nif(ErlNifEnv* env, int argc, const ERL_NIF_TERM argv[]) {",
              ]

              let parts = list.append(parts, params |> list.map(fn (param) {
                let Param(name:, typ:) = param
                case typ {
                  todo
                }
                }))

              // TODO impl

              //   unsigned int s_len;
              //   enif_get_string_length(env, argv[0], &s_len, ERL_NIF_LATIN1);
              //   char *s = malloc(s_len + 1);
              //   if (!enif_get_string(env, argv[0], s, s_len + 1, ERL_NIF_LATIN1)) {
              //     return enif_make_badarg(env);
              //   }

              //   int x;
              //   if (!enif_get_int(env, argv[1], &x)) {
              //     return enif_make_badarg(env);
              //   }

              //   // TODO handle error response for bad input, return an Ok or Error
              //     int result = foobar(s, x);

              //   free(s);

              //   return enif_make_int(env, result);

              let parts = list.append(parts, ["}"])

              parts |> string.join("\n")
            }
          }
        }),
    )

  Ok(parts |> list.map(undent) |> string.join("\n\n"))
}

fn undent(s: String) -> String {
  let lines = string.split(s, "\n")
  case lines |> list.map(string.first) |> set.from_list |> set.to_list {
    [Ok(" ")] | [Ok("\t")] ->
      undent(
        lines
        |> list.map(fn(line) { string.drop_start(line, 1) })
        |> string.join("\n"),
      )
    _ -> lines |> string.join("\n")
  }
}
