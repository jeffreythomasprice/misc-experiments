namespace Robowar.Graphics;

using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Silk.NET.OpenGL;

// TODO a kind of reflection based vertex attribute

public class VertexAttributeSpecification<T> where T : unmanaged
{
	public readonly int Size;
	public readonly VertexAttribPointerType Type;
	public readonly bool Normalized;
	public readonly nint Offset;

	public VertexAttributeSpecification(
		int size,
		VertexAttribPointerType type,
		bool normalized,
		nint offset
	)
	{
		this.Size = size;
		this.Type = type;
		this.Normalized = normalized;
		this.Offset = offset;
	}

	public VertexAttributeSpecification(
		int size,
		VertexAttribPointerType type,
		bool normalized,
		string fieldName
	) : this(size, type, normalized, Marshal.OffsetOf<T>(fieldName)) { }
}

public class VertexSpecification<T> where T : unmanaged
{
	public readonly uint Stride;
	public readonly IReadOnlyDictionary<uint, VertexAttributeSpecification<T>> Attributes;

	public VertexSpecification(IDictionary<uint, VertexAttributeSpecification<T>> attributes)
	{
		Stride = (uint)Unsafe.SizeOf<T>();
		this.Attributes = new Dictionary<uint, VertexAttributeSpecification<T>>(attributes);
	}
}

public class VertexArray<T> : IDisposable where T : unmanaged
{
	private readonly GL gl;
	private readonly VertexSpecification<T> vertexSpecification;
	private readonly uint vertexArray;
	private readonly int verticesLength;
	private readonly uint arrayBuffer;
	private readonly int indicesLength;
	private readonly uint elementArrayBuffer;

	public VertexArray(
		GL gl,
		VertexSpecification<T> vertexSpecification,
		ReadOnlySpan<T> vertices,
		BufferUsageARB verticesUsage,
		ReadOnlySpan<UInt16> indices,
		BufferUsageARB indicesUsage
	)
	{
		this.gl = gl;
		this.vertexSpecification = vertexSpecification;

		vertexArray = gl.GenVertexArray();
		gl.BindVertexArray(vertexArray);
		verticesLength = vertices.Length;
		arrayBuffer = gl.GenBuffer();
		gl.BindBuffer(BufferTargetARB.ArrayBuffer, arrayBuffer);
		unsafe
		{
			fixed (void* p = &vertices.GetPinnableReference())
			{
				gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * Unsafe.SizeOf<T>()), p, verticesUsage);
			}
		}
		indicesLength = indices.Length;
		elementArrayBuffer = gl.GenBuffer();
		gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, elementArrayBuffer);
		gl.BufferData<UInt16>(BufferTargetARB.ElementArrayBuffer, indices, indicesUsage);

		foreach (var (index, attribute) in vertexSpecification.Attributes)
		{
			gl.VertexAttribPointer(index, attribute.Size, attribute.Type, attribute.Normalized, vertexSpecification.Stride, attribute.Offset);
			gl.EnableVertexAttribArray(index);
		}
	}

	public void Dispose()
	{
		gl.DeleteVertexArray(vertexArray);
		gl.DeleteBuffer(arrayBuffer);
		gl.DeleteBuffer(elementArrayBuffer);
	}

	public void Bind()
	{
		gl.BindVertexArray(vertexArray);
	}

	public uint Stride => vertexSpecification.Stride;

	public int VerticesLength => verticesLength;

	public int IndicesLength => indicesLength;
}