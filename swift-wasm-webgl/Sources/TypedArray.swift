import JavaScriptKit

class TypedArray<T: TypedArraySerialization> {
    let elementLengthInBytes: Int

    private var data: JSTypedArray<UInt8>
    private var _length: Int

    init(capacity: Int) {
        let (_, len) = T.readFrom(source: JSTypedArray<T.T>(length: MemoryLayout<T>.size), offset: 0)
        elementLengthInBytes = len * MemoryLayout<T.T>.size

        data = JSTypedArray<UInt8>(length: capacity * elementLengthInBytes)
        _length = 0
    }

    convenience init(array: [T]) {
        self.init(capacity: array.count)
        for elem in array {
            append(elem)
        }
    }

    var capacity: Int {
        get { data.lengthInBytes / elementLengthInBytes }
        set {
            if newValue != capacity {
                let newData = JSTypedArray<UInt8>(length: newValue * elementLengthInBytes)
                _ = newData.jsValue.set(data)
                data = newData
            }
        }
    }

    var length: Int { self._length }

    var lengthInBytes: Int { length * elementLengthInBytes }

    var buffer: JSTypedArray<UInt8> { data }

    subscript(index: Int) -> T {
        get {
            let (result, _) = T.readFromU8(source: data, offset: index * elementLengthInBytes)
            return result
        }
        set {
            _ = newValue.writeToU8(destination: data, offset: index * elementLengthInBytes)
        }
    }

    func append(_ elem: T) {
        if length == capacity {
            capacity += 1
        }
        _ = elem.writeToU8(destination: data, offset: length * elementLengthInBytes)
        _length += 1
    }

    // TODO iterator? foreach?
}

/*
TODO tests

let a = TypedArray<Vector2<Float32>>(capacity: 0)
a.append(Vector2(x: 1, y: 2))
a.append(Vector2(x: 3, y: 4))
a.append(Vector2(x: 5, y: 6))
print("a.length = \(a.length)")
print("a.capacity = \(a.capacity)")
print("a[0] = \(a[0])")
print("a[1] = \(a[1])")
print("a[2] = \(a[2])")
a[1] = Vector2(x: 42, y: 43)
print("a[0] = \(a[0])")
print("a[1] = \(a[1])")
print("a[2] = \(a[2])")
*/
