import JavaScriptKit

class WebGLBuffer<T: TypedArraySerialization> {
    enum BufferUsage {
        case staticDraw
        case dynamicDraw
        case streamDraw
        case staticRead
        case dynamicRead
        case streamRead
        case staticCopy
        case dynamicCopy
        case streamCopy

        func glValue(gl: JSValue) -> Int {
            switch self {
            case .staticDraw:
                return Int(gl.STATIC_DRAW.number!)
            case .dynamicDraw:
                return Int(gl.DYNAMIC_DRAW.number!)
            case .streamDraw:
                return Int(gl.STREAM_DRAW.number!)
            case .staticRead:
                return Int(gl.STATIC_READ.number!)
            case .dynamicRead:
                return Int(gl.DYNAMIC_READ.number!)
            case .streamRead:
                return Int(gl.STREAM_READ.number!)
            case .staticCopy:
                return Int(gl.STATIC_COPPY.number!)
            case .dynamicCopy:
                return Int(gl.DYNAMIC_COPY.number!)
            case .streamCopy:
                return Int(gl.STREAM_COPY.number!)
            }
        }
    }

    enum BufferType {
        case array
        case elementArray

        func glValue(gl: JSValue) -> Int {
            switch self {
            case .array:
                return Int(gl.ARRAY_BUFFER.number!)
            case .elementArray:
                return Int(gl.ELEMENT_ARRAY_BUFFER.number!)
            }
        }
    }

    private let gl: JSValue
    private let type: BufferType
    private let usage: BufferUsage
    private let jsBuffer: TypedArray<T>
    private let webglBuffer: JSValue
    private let dirty: DirtyRegion

    private init(
        gl: JSValue,
        type: BufferType,
        usage: BufferUsage,
        capacity: Int
    ) {
        self.gl = gl
        self.type = type
        self.usage = usage

        jsBuffer = TypedArray(capacity: capacity)

        webglBuffer = gl.createBuffer()
        _ = gl.bindBuffer(type.glValue(gl: gl), webglBuffer)
        _ = gl.bufferData(type.glValue(gl: gl), jsBuffer.capacityInBytes, usage.glValue(gl: gl))
        _ = gl.bindBuffer(type.glValue(gl: gl), JSValue.null)

        dirty = DirtyRegion()
    }

    convenience init(
        gl: JSValue,
        type: BufferType,
        usage: BufferUsage,
        sequence: any Sequence<T>
    ) {
        self.init(gl: gl, type: type, usage: usage, capacity: 0)
        append(sequence)
    }

    convenience init(
        gl: JSValue,
        type: BufferType,
        usage: BufferUsage,
        collection: any Collection<T>
    ) {
        self.init(gl: gl, type: type, usage: usage, capacity: 0)
        append(collection)
    }

    // TODO dispose?

    var capacity: Int {
        get { jsBuffer.capacity }
        set {
            if newValue != jsBuffer.capacity {
                jsBuffer.capacity = newValue

                _ = gl.bindBuffer(type.glValue(gl: gl), webglBuffer)
                _ = gl.bufferData(type.glValue(gl: gl), jsBuffer.capacityInBytes, usage.glValue(gl: gl))
                _ = gl.bindBuffer(type.glValue(gl: gl), JSValue.null)

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
        _ = gl.bindBuffer(type.glValue(gl: gl), webglBuffer)
        if dirty.isDirty {
            flushAssumeBound()
        }
    }

    func flush() {
        _ = gl.bindBuffer(type.glValue(gl: gl), webglBuffer)
        flushAssumeBound()
        _ = gl.bindBuffer(type.glValue(gl: gl), JSValue.null)
    }

    private func flushAssumeBound() {
        for range in dirty.clear() {
            _ = gl.bufferSubData(
                type.glValue(gl: gl),
                range.index * jsBuffer.elementLengthInBytes,
                jsBuffer.buffer,
                range.index * jsBuffer.elementLengthInBytes,
                range.count * jsBuffer.elementLengthInBytes
            )
        }
    }
}

extension WebGLBuffer: Sequence {
    typealias Element = T

    typealias Iterator = TypedArray<T>.Iterator

    func makeIterator() -> TypedArray<T>.Iterator {
        jsBuffer.makeIterator()
    }
}

extension WebGLBuffer: Collection {
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
