namespace Experiment.Graphics

open System
open System.Runtime.InteropServices
open System.Runtime.CompilerServices
open Silk.NET.OpenGL

type VertexAttributeSpecification =
    { Size: int
      Type: VertexAttribPointerType
      Normalized: bool
      Offset: nativeint }

    static member FromFieldName<'T>
        (size: int)
        (``type``: VertexAttribPointerType)
        (normalized: bool)
        (fieldName: string)
        =
        let offset = Marshal.OffsetOf<'T> fieldName

        { Size = size
          Type = ``type``
          Normalized = normalized
          Offset = offset }

type VertexSpecification<'T> =
    { Attributes: Map<uint32, VertexAttributeSpecification> }

    member this.Stride = uint32 (Unsafe.SizeOf<'T>())

type VertexArray<'T when 'T: unmanaged and 'T: (new: unit -> 'T) and 'T: struct and 'T :> ValueType>
    private
    (
        gl: GL,
        vertexSpecification: VertexSpecification<'T>,
        vertexArray: uint32,
        arrayBuffer: uint32,
        elementArrayBuffer: uint32,
        verticesLength: int,
        indicesLength: int
    ) =
    static member New
        (
            gl: GL,
            vertexSpecification: VertexSpecification<'T>,
            vertices: ReadOnlySpan<'T>,
            verticesUsage: BufferUsageARB,
            indices: ReadOnlySpan<uint16>,
            indicesUsage: BufferUsageARB
        ) =
        let vertexArray = gl.GenVertexArray()
        gl.BindVertexArray vertexArray
        let arrayBuffer = gl.GenBuffer()
        gl.BindBuffer(BufferTargetARB.ArrayBuffer, arrayBuffer)
        gl.BufferData<'T>(BufferTargetARB.ArrayBuffer, vertices, verticesUsage)
        let elementArrayBuffer = gl.GenBuffer()
        gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, elementArrayBuffer)
        gl.BufferData<uint16>(BufferTargetARB.ElementArrayBuffer, indices, indicesUsage)

        vertexSpecification.Attributes
        |> Map.iter (fun index attribute ->
            gl.VertexAttribPointer(
                index,
                attribute.Size,
                attribute.Type,
                attribute.Normalized,
                vertexSpecification.Stride,
                attribute.Offset
            )

            gl.EnableVertexAttribArray index)

        new VertexArray<'T>(
            gl,
            vertexSpecification,
            vertexArray,
            arrayBuffer,
            elementArrayBuffer,
            vertices.Length,
            indices.Length
        )

    interface IDisposable with
        member this.Dispose() : unit =
            gl.DeleteVertexArray vertexArray
            gl.DeleteBuffer arrayBuffer
            gl.DeleteBuffer elementArrayBuffer

    member this.Bind() = gl.BindVertexArray vertexArray

    member this.Stride = vertexSpecification.Stride

    member this.VerticesLength = verticesLength

    member this.IndicesLength = indicesLength
