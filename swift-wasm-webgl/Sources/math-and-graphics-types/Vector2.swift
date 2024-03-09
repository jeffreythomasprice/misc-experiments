import JavaScriptKit

struct Vector2<T: TypedArrayElement> {
    let x: T
    let y: T
}

extension Vector2: TypedArraySerialization {
    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        destination[offset] = x
        offset += 1
        destination[offset] = y
        offset += 1
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (Vector2<T>, Int) {
        var offset = offset
        let x = source[offset]
        offset += 1
        let y = source[offset]
        offset += 1
        return (Vector2(x: x, y: y), offset)
    }
}
