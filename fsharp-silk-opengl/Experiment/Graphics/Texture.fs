namespace Experiment.Graphics

open System
open System.IO
open Silk.NET.OpenGL
open Silk.NET.Maths
open StbImageSharp

type Texture private (gl: GL, texture: uint32, width: uint32, height: uint32) =
    static member NewFromImage (gl: GL) (image: ImageResult) =
        let texture = gl.GenTexture()
        gl.BindTexture(TextureTarget.Texture2D, texture)
        // TODO if not power of 2 do clamp?
        gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureMagFilter, int TextureMagFilter.Linear)
        gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureMinFilter, int TextureMagFilter.Nearest)
        gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureWrapS, int TextureWrapMode.Repeat)
        gl.TexParameter(TextureTarget.Texture2D, TextureParameterName.TextureWrapT, int TextureWrapMode.Repeat)

        gl.TexImage2D(
            TextureTarget.Texture2D,
            0,
            InternalFormat.Rgba,
            uint32 image.Width,
            uint32 image.Height,
            0,
            PixelFormat.Rgba,
            PixelType.UnsignedByte,
            ReadOnlySpan image.Data
        )

        new Texture(gl, texture, uint32 image.Width, uint32 image.Height)

    interface IDisposable with
        member this.Dispose() : unit = gl.DeleteTexture texture

    static member NewFromStream (gl: GL) (stream: Stream) =
        let image = ImageResult.FromStream(stream, ColorComponents.RedGreenBlueAlpha)
        Texture.NewFromImage gl image

    member this.Width = width
    member this.Height = height
    member this.Size = new Vector2D<uint32>(width, height)

    member this.Bind() =
        gl.BindTexture(TextureTarget.Texture2D, texture)
