namespace Experiment

module ManifestResources =
    open System.Reflection
    open System.IO
    open Silk.NET.OpenGL
    open Experiment.Graphics

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

    let loadShaderFromManifestResources
        (gl: GL)
        (assembly: Assembly)
        (vertexShaderManifestResourceName: string)
        (fragmentShaderManifestResourceName: string)
        =
        let vertexShaderSource =
            manifestResourceString assembly vertexShaderManifestResourceName
            |> Result.mapError (fun e -> $"error loading vertex shader: {e}")

        let fragmentShaderSource =
            manifestResourceString assembly fragmentShaderManifestResourceName
            |> Result.mapError (fun e -> $"error loading fragment shader: {e}")

        match vertexShaderSource, fragmentShaderSource with
        | Ok vertexShaderSource, Ok fragmentShaderSource ->
            Shader.New gl vertexShaderSource fragmentShaderSource
            |> Result.mapError (fun e -> [ $"error creating shader: {e}" ])
        | Error vertexShaderError, Ok _ -> Error [ vertexShaderError ]
        | Ok _, Error fragmentShaderError -> Error [ fragmentShaderError ]
        | Error vertexShaderError, Error fragmentShaderError -> Error [ vertexShaderError; fragmentShaderError ]

    let loadTextureFromManifestResource (gl: GL) (assembly: Assembly) (manifestResourceName: string) =
        match manifestResourceStream assembly manifestResourceName with
        | Ok stream ->
            use stream = stream
            Ok(Texture.NewFromStream gl stream)
        | Error e -> Error $"error loading texture: {e}"
