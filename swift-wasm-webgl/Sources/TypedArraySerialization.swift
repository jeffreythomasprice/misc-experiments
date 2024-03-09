import JavaScriptKit

protocol TypedArraySerialization {
    associatedtype T: TypedArrayElement

    func WriteTo(destination: JSTypedArray<T>, offset: Int) -> Int
    static func ReadFrom(source: JSTypedArray<T>, offset: Int) -> (Self, Int)
}

extension TypedArraySerialization {
    // TODO WriteToU8

    // TODO ReadFromU8
    // static func ReadFromU8(source: JSTypedArray<UInt8>, offset: Int) -> Self {
    // 	// new Float32Array(new Uint8Array(8).buffer, 4)
    // 	// JSTypedArray(unsafelyWrapping: JSObject
    // 	JSTypedArray<T>.
    // }
}

extension Array: TypedArraySerialization where Element: TypedArraySerialization {
    typealias T = Element.T

    func WriteTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        for next in self {
            offset = next.WriteTo(destination: destination, offset: offset)
        }
        return offset
    }

    static func ReadFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> ([Element], Int) {
        var offset = offset
        // TODO preallocate so we don't resize all the time
        var results = Self()
        while true {
            var next: Element
            (next, offset) = Element.ReadFrom(source: source, offset: offset)
            results.append(next)
        }
        return (results, offset)
    }
}
