import JavaScriptKit

class TypedArray<T: TypedArraySerialization & StaticSized> {
    private var data: JSTypedArray<UInt8>
    private var _count: Int

    init(capacity: Int) {
        data = JSTypedArray<UInt8>(length: capacity * T.lengthInBytes)
        _count = 0
    }

    convenience init(sequence: any Sequence<T>) {
        self.init(capacity: 0)
        append(sequence)
    }

    convenience init(collection: any Collection<T>) {
        self.init(capacity: 0)
        append(collection)
    }

    var capacity: Int {
        get { data.lengthInBytes / T.lengthInBytes }
        set {
            if newValue != capacity {
                let newData = JSTypedArray<UInt8>(length: newValue * T.lengthInBytes)
                _ = newData.jsValue.set(data)
                data = newData
            }
        }
    }

    var count: Int { self._count }

    var capacityInBytes: Int { capacity * T.lengthInBytes }

    var lengthInBytes: Int { count * T.lengthInBytes }

    var buffer: JSTypedArray<UInt8> { data }

    subscript(index: Int) -> T {
        get {
            let (result, _) = T.readFromU8(source: data, offset: index * T.lengthInBytes)
            return result
        }
        set {
            _ = newValue.writeToU8(destination: data, offset: index * T.lengthInBytes)
        }
    }

    func append(_ elem: T) {
        if count == capacity {
            capacity += 1
        }
        _ = elem.writeToU8(destination: data, offset: lengthInBytes)
        _count += 1
    }

    func append(_ source: any Sequence<T>) {
        for elem in source {
            append(elem)
        }
    }

    func append(_ source: any Collection<T>) {
        capacity = Swift.max(capacity, count + source.count)
        for elem in source {
            append(elem)
        }
    }
}

extension TypedArray: Sequence {
    typealias Element = T

    class Iterator: IteratorProtocol {
        typealias Element = T

        private let source: TypedArray<T>
        private var index: Int

        init(source: TypedArray<T>) {
            self.source = source
            index = 0
        }

        func next() -> T? {
            if index >= source.count {
                return .none
            } else {
                let result = source[index]
                index += 1
                return .some(result)
            }
        }
    }

    func makeIterator() -> Iterator {
        Iterator(source: self)
    }
}

extension TypedArray: Collection {
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
