namespace Experiment.WebGPU;

using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

public record class VertexDescription(int Stride, Silk.NET.WebGPU.VertexAttribute[] Attributes)
{
	public static VertexDescription Create<T>()
	{
		var stride = Unsafe.SizeOf<T>();
		Console.WriteLine($"vertex stride: {stride}");
		var attributes = new List<Silk.NET.WebGPU.VertexAttribute>();
		foreach (var field in typeof(T).GetFields())
		{
			var attr = field.GetCustomAttribute<VertexAttribute>();
			if (attr != null)
			{
				var offset = Marshal.OffsetOf<T>(field.Name);
				Console.WriteLine($"vertex attribute {field}, format={attr.Format}, offset={offset}, shaderLocation={attr.ShaderLocation}");
				attributes.Add(new Silk.NET.WebGPU.VertexAttribute()
				{
					Format = attr.Format,
					Offset = (ulong)offset,
					ShaderLocation = attr.ShaderLocation,
				});
			}
		}
		return new(stride, attributes.ToArray());
	}
}
