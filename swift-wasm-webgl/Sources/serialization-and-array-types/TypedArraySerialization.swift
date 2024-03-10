import JavaScriptKit

protocol StaticSized {
    static var lengthInBytes: Int { get }
}

protocol DynamicSized {
    var lengthInBytes: Int { get }
}

protocol TypedArraySerialization {
    associatedtype T: TypedArrayElement & StaticSized

    func writeTo(destination: JSTypedArray<T>, offset: Int) -> Int
    static func readFrom(source: JSTypedArray<T>, offset: Int) -> (Self, Int)
}

extension TypedArraySerialization {
    func writeToU8(destination: JSTypedArray<UInt8>, offset: Int) -> Int {
        // invoke the raw constructor, since there isn't a well typed wrapper constructor for this form)")
        let typedDestination = JSTypedArray<T>(
            from: JSTypedArray<T>.constructor!.new(
                // drill down the raw buffer
                destination.jsObject.buffer,
                // the offset of the input in that buffer, plus any additional offset we've been given
                Int(destination.jsObject.byteOffset.number!) + offset,
                // the remaining size of the input, rounded down to the nearest multiple of the data size of this type
                (destination.lengthInBytes - offset) / T.lengthInBytes
            )
        )!
        let lengthInTypedElements = writeTo(destination: typedDestination, offset: 0)
        return lengthInTypedElements * T.lengthInBytes
    }

    static func readFromU8(source: JSTypedArray<UInt8>, offset: Int) -> (Self, Int) {
        // see comments in the above
        let typedSource = JSTypedArray<T>(
            from: JSTypedArray<T>.constructor!.new(
                source.jsObject.buffer,
                Int(source.jsObject.byteOffset.number!) + offset,
                (source.lengthInBytes - offset) / T.lengthInBytes
            )
        )!
        let (result, lengthInTypedElements) = readFrom(source: typedSource, offset: 0)
        return (result, lengthInTypedElements * T.lengthInBytes)
    }
}

extension Array: TypedArraySerialization & DynamicSized where Element: TypedArraySerialization {
    typealias T = Element.T

    var lengthInBytes: Int { count * T.lengthInBytes }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        for next in self {
            offset = next.writeTo(destination: destination, offset: offset)
        }
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> ([Element], Int) {
        var offset = offset
        var results = Self()
        results.reserveCapacity(source.lengthInBytes / T.lengthInBytes)
        while true {
            var next: Element
            (next, offset) = Element.readFrom(source: source, offset: offset)
            results.append(next)
        }
        return (results, offset)
    }
}

extension Int8: TypedArraySerialization & StaticSized {
    typealias T = Int8

    static var lengthInBytes: Int { 1 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension UInt8: TypedArraySerialization & StaticSized {
    typealias T = UInt8

    static var lengthInBytes: Int { 1 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension Int16: TypedArraySerialization & StaticSized {
    typealias T = Int16

    static var lengthInBytes: Int { 2 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension UInt16: TypedArraySerialization & StaticSized {
    typealias T = UInt16

    static var lengthInBytes: Int { 2 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension Int32: TypedArraySerialization & StaticSized {
    typealias T = Int32

    static var lengthInBytes: Int { 4 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension UInt32: TypedArraySerialization & StaticSized {
    typealias T = UInt32

    static var lengthInBytes: Int { 4 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension Float32: TypedArraySerialization & StaticSized {
    typealias T = Float32

    static var lengthInBytes: Int { 4 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}

extension Float64: TypedArraySerialization & StaticSized {
    typealias T = Float64

    static var lengthInBytes: Int { 8 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        destination[offset] = self
        return offset + 1
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (T, Int) {
        (source[offset], offset + 1)
    }
}
