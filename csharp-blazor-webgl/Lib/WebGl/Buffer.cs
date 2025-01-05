using System.Collections;
using System.Runtime.InteropServices;

namespace BlazorExperiments.Lib.WebGl;

public class Buffer<T> : IDisposable, IList<T>
{
    private class Enumerator : IEnumerator<T>
    {
        private readonly Buffer<T> buffer;
        private int index;

        public Enumerator(Buffer<T> buffer)
        {
            this.buffer = buffer;
            index = -1;
        }

        public T Current => buffer[index];

        object IEnumerator.Current => Current!;

        public void Dispose() { }

        public bool MoveNext()
        {
            if (index >= buffer.Count)
            {
                return false;
            }
            index++;
            return index < buffer.Count;
        }

        public void Reset()
        {
            index = -1;
        }
    }

    public readonly int Stride = Marshal.SizeOf<T>();

    public readonly WebGL2RenderingContext.BufferType Type;
    public readonly WebGL2RenderingContext.BufferUsage Usage;

    private readonly WebGL2RenderingContext gl;

    private byte[] data;
    private int count;

    private bool disposedValue;

    private readonly WebGL2RenderingContext.Buffer buffer;
    private SparseIntegerSet dirty;

    public Buffer(
        WebGL2RenderingContext gl,
        WebGL2RenderingContext.BufferType type,
        WebGL2RenderingContext.BufferUsage usage,
        int length = 0,
        int capacity = 0
    )
    {
        this.gl = gl;
        this.Type = type;
        this.Usage = usage;

        ArgumentOutOfRangeException.ThrowIfNegative(length);
        ArgumentOutOfRangeException.ThrowIfNegative(capacity);

        if (length > capacity)
        {
            capacity = length;
        }

        this.count = length;
        data = new byte[Stride * capacity];

        buffer = gl.CreateBuffer();
        gl.BindBuffer(type, buffer);
        gl.BufferData(type, data.Length, usage);
        gl.BindBuffer(type, null);

        dirty = new SparseIntegerSet();
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!disposedValue)
        {
            gl.DeleteBuffer(buffer);

            disposedValue = true;
        }
    }

    ~Buffer()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: false);
    }

    public void Dispose()
    {
        // Do not change this code. Put cleanup code in 'Dispose(bool disposing)' method
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }

    public void Bind()
    {
        gl.BindBuffer(Type, buffer);

        if (!dirty.IsEmpty)
        {
            foreach (var range in dirty.Ranges)
            {
                var start = range.Start.Value * Stride;
                var end = range.End.Value * Stride;
                gl.BufferSubData(Type, start, data[start..end]);
            }
            dirty.Clear();
        }
    }

    public int Capacity
    {
        get
        {
            return data.Length / Stride;
        }
        set
        {
            ArgumentOutOfRangeException.ThrowIfNegative(value);
            if (Capacity != value)
            {
                count = Math.Min(count, value);

                var newData = new byte[Stride * value];
                Array.Copy(data, newData, count * Stride);
                data = newData;

                gl.BindBuffer(Type, buffer);
                gl.BufferData(Type, data.Length, Usage);
                gl.BindBuffer(Type, null);

                dirty.Clear();
                dirty.Add(0..Count);
            }
        }
    }

    public int Count
    {
        get
        {
            return count;
        }
        set
        {
            ArgumentOutOfRangeException.ThrowIfNegative(value);
            if (count != value)
            {
                if (value > count)
                {
                    dirty.Add(count..value);
                }

                Capacity = Math.Max(Capacity, value);
                count = value;
            }
        }
    }

    public T this[int index]
    {
        get
        {
            if (index < 0 || index >= Count)
            {
                throw new ArgumentOutOfRangeException(nameof(index));
            }
            unsafe
            {
                fixed (byte* ptr = &data[index * Stride])
                {
                    return Marshal.PtrToStructure<T>((nint)ptr)!;
                }
            }
        }
        set
        {
            if (index < 0 || index >= Count)
            {
                throw new ArgumentOutOfRangeException(nameof(index));
            }
            unsafe
            {
                fixed (byte* ptr = &data[index * Stride])
                {
                    Marshal.StructureToPtr(value!, (nint)ptr, false);
                }
            }
            dirty.Add(index);
        }
    }

    public bool IsReadOnly => false;

    public void Add(T item)
    {
        Count++;
        this[Count - 1] = item;
    }

    public void Clear()
    {
        Count = 0;
    }

    public bool Contains(T item)
    {
        foreach (var x in this)
        {
            if (item?.Equals(x) == true)
            {
                return true;
            }
        }
        return false;
    }

    public void CopyTo(T[] array, int arrayIndex)
    {
        for (var i = arrayIndex; i < array.Length; i++)
        {
            var j = i - arrayIndex;
            if (j >= Count)
            {
                break;
            }
            array[i] = this[i - arrayIndex];
        }
    }

    public int IndexOf(T item)
    {
        for (var i = 0; i < Count; i++)
        {
            if (item?.Equals(this[i]) == true)
            {
                return i;
            }
        }
        return -1;
    }

    public void Insert(int index, T item)
    {
        if (index < 0 || index > Count)
        {
            throw new ArgumentOutOfRangeException(nameof(index));
        }
        Count++;
        for (var i = Count - 1; i > index; i--)
        {
            this[i] = this[i - 1];
        }
        this[index] = item;
    }

    public bool Remove(T item)
    {
        var i = IndexOf(item);
        if (i >= 0)
        {
            RemoveAt(i);
            return true;
        }
        else
        {
            return false;
        }
    }

    public void RemoveAt(int index)
    {
        if (index < 0 || index >= Count)
        {
            throw new ArgumentOutOfRangeException(nameof(index));
        }
        for (var i = index; i + 1 < Count; i++)
        {
            this[i] = this[i + 1];
        }
        Count--;
    }

    public IEnumerator<T> GetEnumerator()
    {
        return new Enumerator(this);
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return GetEnumerator();
    }
}
