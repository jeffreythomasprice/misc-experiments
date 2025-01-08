namespace BlazorExperiments.Lib.WebGl;

public class VertexAttribute : Attribute
{
    public readonly string? Name;
    public readonly int Size;
    public readonly WebGL2RenderingContext.DataType DataType;
    public readonly bool Normalized;

    public VertexAttribute(int size, WebGL2RenderingContext.DataType dataType, bool normalized) : this(null, size, dataType, normalized) { }

    public VertexAttribute(string? name, int size, WebGL2RenderingContext.DataType dataType, bool normalized)
    {
        Name = name;
        Size = size;
        DataType = dataType;
        Normalized = normalized;
    }
}