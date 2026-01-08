namespace Experiment.Engine;

using System.Runtime.CompilerServices;
using Experiment.VulkanUtils;
using Silk.NET.Vulkan;

public sealed class Mesh<VertexType> : IDisposable
{
    private readonly Vk vk;
    private readonly BufferWrapper<VertexType> vertexBuffer;
    private int vertexBufferCount;
    private readonly BufferWrapper<UInt16> indexBuffer;
    private int indexBufferCount;

    public Mesh(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        Span<VertexType> vertices,
        Span<UInt16> indices
    )
    {
        this.vk = vk;
        vertexBuffer = new BufferWrapper<VertexType>(
            vk,
            physicalDevice,
            device,
            vertices,
            BufferUsageFlags.VertexBufferBit
        );
        vertexBufferCount = vertices.Length;
        indexBuffer = new BufferWrapper<UInt16>(
            vk,
            physicalDevice,
            device,
            indices,
            BufferUsageFlags.IndexBufferBit
        );
        indexBufferCount = indices.Length;
    }

    public Mesh(
        Vk vk,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        int vertexCount,
        int indexCount
    )
    {
        this.vk = vk;
        vertexBuffer = new BufferWrapper<VertexType>(
            vk,
            physicalDevice,
            device,
            vertexCount,
            BufferUsageFlags.VertexBufferBit
        );
        indexBuffer = new BufferWrapper<UInt16>(
            vk,
            physicalDevice,
            device,
            indexCount,
            BufferUsageFlags.IndexBufferBit
        );
    }

    public void Dispose()
    {
        vertexBuffer.Dispose();
        indexBuffer.Dispose();
    }

    public int VertexBufferCapacity => vertexBuffer.Count;
    public int VertexBufferCount => vertexBufferCount;

    public int IndexBufferCapacity => indexBuffer.Count;
    public int IndexBufferCount => indexBufferCount;

    public void Append(Span<VertexType> vertices, Span<UInt16> indices)
    {
        if (vertexBufferCount + vertices.Length > vertexBuffer.Count)
        {
            throw new InvalidOperationException(
                "not enough space in vertex buffer to append vertices"
            );
        }
        if (indexBufferCount + indices.Length > indexBuffer.Count)
        {
            throw new InvalidOperationException(
                "not enough space in index buffer to append indices"
            );
        }

        var lastVertex = vertexBufferCount;
        vertexBuffer.CopyDataToBuffer(vertices, lastVertex);
        vertexBufferCount += vertices.Length;

        var indicesPlusOffset = new UInt16[indices.Length];
        for (var i = 0; i < indices.Length; i++)
        {
            indicesPlusOffset[i] = (UInt16)(indices[i] + lastVertex);
        }
        indexBuffer.CopyDataToBuffer(indicesPlusOffset, indexBufferCount);
        indexBufferCount += indices.Length;
    }

    public void AppendTriangle(VertexType a, VertexType b, VertexType c)
    {
        Append([a, b, c], [0, 1, 2]);
    }

    public void AppendTriangleFan(Span<VertexType> vertices)
    {
        if (vertices.Length < 3)
        {
            throw new ArgumentException(
                "need at least 3 vertices for triangle fan",
                nameof(vertices)
            );
        }

        var indices = new UInt16[(vertices.Length - 2) * 3];
        for (var i = 0; i < vertices.Length - 2; i++)
        {
            indices[i * 3 + 0] = 0;
            indices[i * 3 + 1] = (UInt16)(i + 1);
            indices[i * 3 + 2] = (UInt16)(i + 2);
        }

        Append(vertices, indices);
    }

    public void AppendQuad(VertexType a, VertexType b, VertexType c, VertexType d)
    {
        AppendTriangleFan([a, b, c, d]);
    }

    /// <param name="commandBuffer"></param>
    /// <param name="offset">in indices, not bytes</param>
    /// <param name="count">in indices, not bytes</param>
    public void BindAndDraw(CommandBufferWrapper commandBuffer, int offset, int count)
    {
        vk.CmdBindVertexBuffers(commandBuffer.CommandBuffer, 0, 1, [vertexBuffer.Buffer], [0]);
        vk.CmdBindIndexBuffer(commandBuffer.CommandBuffer, indexBuffer.Buffer, 0, IndexType.Uint16);
        vk.CmdDrawIndexed(commandBuffer.CommandBuffer, (uint)count, 1, (uint)offset, 0, 0);
    }

    public void BindAndDraw(CommandBufferWrapper commandBuffer)
    {
        BindAndDraw(commandBuffer, 0, indexBufferCount);
    }
}
