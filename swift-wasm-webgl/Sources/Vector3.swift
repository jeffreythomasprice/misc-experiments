import JavaScriptKit

struct Vector3<T: TypedArrayElement> {
    let x: T
    let y: T
    let z: T
}

extension Vector3: TypedArraySerialization {
    func WriteTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        destination[offset] = x
        offset += 1
        destination[offset] = y
        offset += 1
        destination[offset] = z
        offset += 1
        return offset
    }

    static func ReadFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (Vector3<T>, Int) {
        var offset = offset
        let x = source[offset]
        offset += 1
        let y = source[offset]
        offset += 1
        let z = source[offset]
        offset += 1
        return (Vector3(x: x, y: y, z: z), offset)
    }
}
