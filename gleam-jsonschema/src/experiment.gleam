import gleam/io
import gleam/json
import gleam/list
import gleam/result
import gleam/string
import gleam/uri
import jsonschema
import path
import simplifile

pub fn main() {
  let assert Ok(paths) = recursive_list_directory(path.new("schemas"))
  let paths =
    paths
    |> list.sort(fn(a, b) {
      string.compare(path.to_string(a), path.to_string(b))
    })

  let jsonschema_options =
    jsonschema.Options(fn(ref) {
      // if it's a url assume we're going to find a similiarly named file locally so just ook at the path
      let ref_path = case uri.parse(ref) {
        Ok(ref_uri) -> ref_uri.path
        Error(_) -> ref
      }
      // try to find that in the list of possible files we found
      let file_contents = case
        paths
        |> list.filter(fn(path) {
          string.ends_with(path.to_string(path), ref_path)
        })
      {
        // there's only one possibility, so use that
        [path] -> simplifile.read(path.to_string(path))
        // no such file
        [] -> {
          io.println_error("no such file for ref: " <> ref)
          Error(simplifile.Enoent)
        }
        // ambiguous, also fail
        possible -> {
          io.println_error(
            "multiple possible ref paths: " <> string.inspect(possible),
          )
          Error(simplifile.Enoent)
        }
      }
      // just hide the error
      file_contents |> result.map_error(fn(_) { Nil })
    })

  paths
  |> list.each(fn(p) {
    let p = path.to_string(p)
    io.println("TODO path = " <> p)

    case string.ends_with(p, ".schema.json") {
      True -> {
        let assert Ok(file_contents) = simplifile.read(p)
        let assert Ok(schema) =
          json.decode(file_contents, jsonschema.parse(_, jsonschema_options))
        io.debug(schema)
        Nil
      }
      False -> Nil
    }
  })
  // let assert Ok(file_contents) =
  //   simplifile.read(
  //     "schemas/json-schema-org-learn-json-schema-examples/location-schema.json",
  //   )
  // let assert Ok(schema) = json.decode(file_contents, jsonschema.parse)
  // io.debug(schema)
}

pub fn recursive_list_directory(
  path: path.Path,
) -> Result(List(path.Path), simplifile.FileError) {
  let path_str = path.to_string(path)
  use is_dir <- result.try(simplifile.is_directory(path_str))
  case is_dir {
    True -> {
      // find the directory contents
      use contents <- result.try(simplifile.read_directory(path_str))
      // fix the paths to be relative to the path we used to get here
      use child_paths <- result.try(
        contents
        |> list.map(fn(path_component) {
          let child_path = path.join(path, path.new(path_component))
          recursive_list_directory(child_path)
        })
        |> result.all,
      )
      // recurse
      use results <- result.try(
        child_paths
        |> list.flatten
        |> list.map(recursive_list_directory)
        |> result.all,
      )
      Ok(results |> list.flatten)
    }
    // not looking at a directory, so this is a file we can use
    False -> Ok([path])
  }
}
