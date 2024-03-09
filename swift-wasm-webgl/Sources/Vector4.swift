import JavaScriptKit

struct Vector4<T: TypedArrayElement> {
    let x: T
    let y: T
    let z: T
    let w: T
}

extension Vector4: TypedArraySerialization {
    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        destination[offset] = x
        offset += 1
        destination[offset] = y
        offset += 1
        destination[offset] = z
        offset += 1
        destination[offset] = w
        offset += 1
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (Vector4<T>, Int) {
        var offset = offset
        let x = source[offset]
        offset += 1
        let y = source[offset]
        offset += 1
        let z = source[offset]
        offset += 1
        let w = source[offset]
        offset += 1
        return (Vector4(x: x, y: y, z: z, w: w), offset)
    }
}
