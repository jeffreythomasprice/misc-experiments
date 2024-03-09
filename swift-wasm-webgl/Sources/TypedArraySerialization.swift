import JavaScriptKit

protocol TypedArraySerialization {
    associatedtype T: TypedArrayElement

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
                (destination.lengthInBytes - offset) / MemoryLayout<T>.size
            )
        )!
        let lengthInTypedElements = writeTo(destination: typedDestination, offset: 0)
        return lengthInTypedElements * MemoryLayout<T>.size
    }

    static func readFromU8(source: JSTypedArray<UInt8>, offset: Int) -> (Self, Int) {
        // see comments in the above
        let typedSource = JSTypedArray<T>(
            from: JSTypedArray<T>.constructor!.new(
                source.jsObject.buffer,
                Int(source.jsObject.byteOffset.number!) + offset,
                (source.lengthInBytes - offset) / MemoryLayout<T>.size
            )
        )!
        let (result, lengthInTypedElements) = readFrom(source: typedSource, offset: 0)
        return (result, lengthInTypedElements * MemoryLayout<T>.size)
    }
}

extension Array: TypedArraySerialization where Element: TypedArraySerialization {
    typealias T = Element.T

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        for next in self {
            offset = next.writeTo(destination: destination, offset: offset)
        }
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> ([Element], Int) {
        var offset = offset
        // TODO preallocate so we don't resize all the time
        var results = Self()
        while true {
            var next: Element
            (next, offset) = Element.readFrom(source: source, offset: offset)
            results.append(next)
        }
        return (results, offset)
    }
}
