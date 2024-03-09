import JavaScriptKit

class ArrayBuffer<T: TypedArraySerialization> {
    private var gl: JSValue
    private var usage: BufferUsage
    private var jsBuffer: TypedArray<T>
    private var webglBuffer: JSValue
    private var dirty: DirtyRegion

    init(gl: JSValue, usage: BufferUsage, capacity: Int) {
        self.gl = gl
        self.usage = usage

        jsBuffer = TypedArray(capacity: capacity)

        webglBuffer = gl.createBuffer()
        _ = gl.bindBuffer(gl.ARRAY_BUFFER, webglBuffer)
        _ = gl.bufferData(gl.ARRAY_BUFFER, jsBuffer.capacityInBytes, usage.glValue(gl: gl))
        _ = gl.bindBuffer(gl.ARRAY_BUFFER, JSValue.null)

        dirty = DirtyRegion()
    }

    convenience init(gl: JSValue, usage: BufferUsage, sequence: any Sequence<T>) {
        self.init(gl: gl, usage: usage, capacity: 0)
        append(sequence)
    }

    convenience init(gl: JSValue, usage: BufferUsage, collection: any Collection<T>) {
        self.init(gl: gl, usage: usage, capacity: 0)
        append(collection)
    }

    var capacity: Int {
        get { jsBuffer.capacity }
        set {
            if newValue != jsBuffer.capacity {
                jsBuffer.capacity = newValue

                _ = gl.bindBuffer(gl.ARRAY_BUFFER, webglBuffer)
                _ = gl.bufferData(gl.ARRAY_BUFFER, jsBuffer.capacityInBytes, usage.glValue(gl: gl))
                _ = gl.bindBuffer(gl.ARRAY_BUFFER, JSValue.null)

                if let range = DirtyRegion.Range(index: 0, count: count) {
                    dirty.append(range: range)
                } else {
                    _ = dirty.clear()
                }
            }
        }
    }

    var count: Int { jsBuffer.count }

    subscript(index: Int) -> T {
        get {
            jsBuffer[index]
        }
        set {
            jsBuffer[index] = newValue
            dirty.append(index: index)
        }
    }

    func append(_ elem: T) {
        if count == capacity {
            capacity += 1
        }
        dirty.append(index: count)
        jsBuffer.append(elem)
    }

    func append(_ source: any Sequence<T>) {
        for elem in source {
            append(elem)
        }
    }

    func append(_ source: any Collection<T>) {
        capacity = Swift.max(capacity, count + source.count)
        let start = count
        jsBuffer.append(source)
        let end = count - 1
        if let d = DirtyRegion.Range(index: start, count: end - start + 1) {
            dirty.append(range: d)
        }
    }

    func bind() {
        _ = gl.bindBuffer(gl.ARRAY_BUFFER, webglBuffer)
        if dirty.isDirty {
            flushAssumeBound()
        }
    }

    func flush() {
        _ = gl.bindBuffer(gl.ARRAY_BUFFER, webglBuffer)
        flushAssumeBound()
        _ = gl.bindBuffer(gl.ARRAY_BUFFER, JSValue.null)
    }

    private func flushAssumeBound() {
        for range in dirty.clear() {
            _ = gl.bufferSubData(
                gl.ARRAY_BUFFER,
                range.index * jsBuffer.elementLengthInBytes,
                jsBuffer.buffer,
                range.index * jsBuffer.elementLengthInBytes,
                range.count * jsBuffer.elementLengthInBytes
            )
        }
    }
}

extension ArrayBuffer: Sequence {
    typealias Element = T

    typealias Iterator = TypedArray<T>.Iterator

    func makeIterator() -> TypedArray<T>.Iterator {
        jsBuffer.makeIterator()
    }
}

extension ArrayBuffer: Collection {
    typealias Index = Int

    var startIndex: Int {
        0
    }

    var endIndex: Int {
        count
    }

    func index(after i: Int) -> Int {
        i + 1
    }
}
