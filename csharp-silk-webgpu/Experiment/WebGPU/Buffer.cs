namespace Experiment.WebGPU;

using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Silk.NET.WebGPU;

public unsafe class Buffer<T> : IDisposable where T : unmanaged
{
	private readonly VideoDriver videoDriver;
	private readonly int stride;
	private readonly int length;
	private readonly int sizeInBytes;
	private readonly Silk.NET.WebGPU.Buffer* buffer;

	public Buffer(VideoDriver videoDriver, ReadOnlySpan<T> data, BufferUsage usage)
	{
		stride = Unsafe.SizeOf<T>();

		this.videoDriver = videoDriver;
		length = data.Length;
		sizeInBytes = length * stride;

		// if you don't have the input data size be a multiple of COPY_BUFFER_ALIGNMENT you get
		// Copy size 6 does not respect `COPY_BUFFER_ALIGNMENT`
		// or whatever the input byte size is
		var paddedLengthInBytes = data.Length * stride;
		if (paddedLengthInBytes % 4 != 0)
		{
			while (paddedLengthInBytes % 4 != 0)
			{
				paddedLengthInBytes += stride;
			}
			var desiredLength = paddedLengthInBytes / stride;
			var newData = new T[desiredLength];
			data.CopyTo(newData);
			data = newData;
		}

		var descriptor = new BufferDescriptor()
		{
			MappedAtCreation = false,
			Size = (ulong)paddedLengthInBytes,
			Usage = BufferUsage.CopyDst | usage,
		};
		buffer = videoDriver.WebGPU.DeviceCreateBuffer(videoDriver.Device, ref descriptor);

		videoDriver.WebGPU.QueueWriteBuffer<T>(videoDriver.Queue, buffer, 0, data, (nuint)paddedLengthInBytes);
	}

	public void Dispose()
	{
		videoDriver.WebGPU.BufferRelease(buffer);
	}

	public int Stride => stride;

	public int Length => length;

	public int SizeInBytes => sizeInBytes;

	public Silk.NET.WebGPU.Buffer* Instance => buffer;

	public void QueueWrite(ReadOnlySpan<T> data, int index)
	{
		var startBytes = index * stride;
		var endBytes = (index + data.Length) * stride;
		if (startBytes > SizeInBytes || endBytes > SizeInBytes)
		{
			throw new IndexOutOfRangeException($"input extends beyond the end of the buffer, buffer size in bytes: {SizeInBytes}, input length: {data.Length}, index = {index}, stride = {stride}");
		}
		videoDriver.WebGPU.QueueWriteBuffer(videoDriver.Queue, buffer, (ulong)startBytes, data, (nuint)(data.Length * stride));
	}
}
