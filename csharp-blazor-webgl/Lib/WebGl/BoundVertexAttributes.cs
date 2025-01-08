using System.Reflection;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Lib.WebGl;

public class BoundVertexAttributes<T>
{
    private record class Item(
        VertexAttribute VertexAttribute,
        Shader.Attribute ShaderAttribute,
        int Offset
    );

    private readonly WebGL2RenderingContext gl;
    private readonly int stride;
    private readonly List<Item> items;

    public BoundVertexAttributes(WebGL2RenderingContext gl, Shader shader)
    {
        this.gl = gl;
        stride = Marshal.SizeOf<T>();

        var items = new List<Item>();
        foreach (var f in typeof(T).GetFields())
        {
            var vertexAttribute = f.GetCustomAttribute<VertexAttribute>();
            if (vertexAttribute != null)
            {
                items.Add(new(
                    vertexAttribute,
                    FindShaderAttribute(shader, vertexAttribute.Name ?? f.Name),
                    (int)Marshal.OffsetOf<T>(f.Name)
                 ));
            }
        }
        this.items = items;

        if (this.items.Count == 0)
        {
            throw new ArgumentException($"{typeof(T)} has no {typeof(VertexAttribute)} attributes");
        }
    }

    public void Enable()
    {
        foreach (var item in items)
        {
            gl.EnableVertexAttribArray(item.ShaderAttribute.Location);
            gl.VertexAttribPointer(
                item.ShaderAttribute.Location,
                item.VertexAttribute.Size,
                item.VertexAttribute.DataType,
                item.VertexAttribute.Normalized,
                stride,
                item.Offset
            );
        }
    }

    public void Disable()
    {
        foreach (var item in items)
        {
            gl.DisableVertexAttribArray(item.ShaderAttribute.Location);
        }
    }

    private static Shader.Attribute FindShaderAttribute(Shader shader, string name)
    {
        var result = shader.Attributes.GetValueOrDefault(name);
        if (result != null)
        {
            return result;
        }
        var possibilities = shader.Attributes.Keys.Where(x => x.Equals(name, StringComparison.OrdinalIgnoreCase)).ToList();
        if (possibilities.Count == 0)
        {
            throw new ArgumentException($"shader has no attribute that matches name {name}");
        }
        if (possibilities.Count > 1)
        {
            throw new ArgumentException($"shader has multiple attributes that matches name {name} with case-insensitive search");
        }
        return shader.Attributes[possibilities[0]];
    }
}