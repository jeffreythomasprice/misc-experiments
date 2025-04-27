using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Silk.NET.OpenGL;

public class VertexAttributeSpecification
{
	public readonly uint Index;
	public readonly int Size;
	public readonly VertexAttribPointerType Type;
	public readonly bool Normalized;
	public readonly uint Stride;
	public readonly nint Offset;

	public VertexAttributeSpecification(
		uint index,
		int size,
		VertexAttribPointerType type,
		bool normalized,
		uint stride,
		nint offset
	)
	{
		this.Index = index;
		this.Size = size;
		this.Type = type;
		this.Normalized = normalized;
		this.Stride = stride;
		this.Offset = offset;
	}
}

public class VertexAttributeSpecification<T> : VertexAttributeSpecification
{
	public VertexAttributeSpecification(
		uint index,
		int size,
		VertexAttribPointerType type,
		bool normalized,
		nint offset
	) : base(index, size, type, normalized, (uint)Unsafe.SizeOf<T>(), offset) { }

	public VertexAttributeSpecification(
		uint index,
		int size,
		VertexAttribPointerType type,
		bool normalized,
		string fieldName
	) : this(index, size, type, normalized, Marshal.OffsetOf<T>(fieldName)) { }
}

public class VertexSpecification
{
	public readonly IReadOnlyList<VertexAttributeSpecification> Attributes;

	public VertexSpecification(IEnumerable<VertexAttributeSpecification> attributes)
	{
		this.Attributes = [.. attributes];
		// TODO throw if any indices overlap
	}
}

public class VertexArray<T> : IDisposable where T : unmanaged
{
	private readonly GL gl;
	private readonly uint vertexArray;
	private readonly uint arrayBuffer;
	private readonly uint elementArrayBuffer;

	public VertexArray(
		GL gl,
		VertexSpecification vertexSpecification,
		ReadOnlySpan<T> vertices,
		BufferUsageARB verticesUsage,
		ReadOnlySpan<UInt16> indices,
		BufferUsageARB indicesUsage
	)
	{
		this.gl = gl;

		vertexArray = gl.GenVertexArray();
		gl.BindVertexArray(vertexArray);
		arrayBuffer = gl.GenBuffer();
		gl.BindBuffer(BufferTargetARB.ArrayBuffer, arrayBuffer);
		unsafe
		{
			fixed (void* p = &vertices.GetPinnableReference())
			{
				gl.BufferData(BufferTargetARB.ArrayBuffer, (nuint)(vertices.Length * Unsafe.SizeOf<T>()), p, verticesUsage);
			}
		}
		elementArrayBuffer = gl.GenBuffer();
		gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, elementArrayBuffer);
		gl.BufferData<UInt16>(BufferTargetARB.ElementArrayBuffer, indices, indicesUsage);

		foreach (var attribute in vertexSpecification.Attributes)
		{
			gl.VertexAttribPointer(attribute.Index, attribute.Size, attribute.Type, attribute.Normalized, attribute.Stride, attribute.Offset);
			gl.EnableVertexAttribArray(attribute.Index);
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
}