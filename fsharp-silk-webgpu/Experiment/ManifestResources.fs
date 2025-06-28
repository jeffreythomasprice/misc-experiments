namespace Experiment

module ManifestResources =
    open System.Reflection
    open System.IO

    let sanitizeName (name: string) =
        name.Replace('/', '*').Replace('-', '*').Replace('_', '*').Replace('.', '*').ToLower()

    let sanitizeManifestResourceName (assembly: Assembly) (name: string) =
        let sanitizedName = sanitizeName name

        match
            assembly.GetManifestResourceNames()
            |> Array.filter (fun resourceName -> sanitizedName = sanitizeName resourceName)
        with
        | [||] -> Error $"no results for {name}"
        | [| result |] -> Ok result
        | _ -> Error $"multiple results for {name}"

    let manifestResourceStream (assembly: Assembly) (name: string) =
        sanitizeManifestResourceName assembly name
        |> Result.bind (fun name ->
            match assembly.GetManifestResourceStream name with
            | null -> Error $"failed to find embedded file: {name}"
            | result -> Ok result)

    let manifestResourceString (assembly: Assembly) (name: string) =
        manifestResourceStream assembly name
        |> Result.bind (fun stream ->
            use stream = stream
            use reader = new StreamReader(stream)
            Ok(reader.ReadToEnd()))
