namespace Experiment.WebGPU;

using Silk.NET.WebGPU;

public unsafe class Buffer<T> : IDisposable where T : unmanaged
{
	private readonly VideoDriver videoDriver;
	private readonly int length;
	private readonly int lengthInBytes;
	private readonly int paddedLength;
	private readonly int paddedLengthInBytes;
	private readonly Silk.NET.WebGPU.Buffer* buffer;

	public Buffer(VideoDriver videoDriver, ReadOnlySpan<T> data, BufferUsage usage)
	{
		this.videoDriver = videoDriver;
		this.length = data.Length;
		this.lengthInBytes = this.length * sizeof(T);

		// if you don't have the input data size be a multiple of COPY_BUFFER_ALIGNMENT you get
		// Copy size 6 does not respect `COPY_BUFFER_ALIGNMENT`
		// or whatever the input byte size is
		var lengthInBytes = data.Length * sizeof(T);
		if (lengthInBytes % 4 != 0)
		{
			while (lengthInBytes % 4 != 0)
			{
				lengthInBytes += sizeof(T);
			}
			var desiredLength = lengthInBytes / sizeof(T);
			var newData = new T[desiredLength];
			data.CopyTo(newData);
			data = newData;
		}
		this.paddedLength = lengthInBytes / sizeof(T);
		this.paddedLengthInBytes = lengthInBytes;

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

	public int Length => length;

	public int SizeInBytes => sizeof(T) * length;

	public Silk.NET.WebGPU.Buffer* Instance => buffer;
}
