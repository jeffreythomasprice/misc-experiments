import JavaScriptKit

struct RGBA<T: TypedArrayElement & StaticSized> {
    let r: T
    let g: T
    let b: T
    let a: T
}

extension RGBA: TypedArraySerialization & StaticSized {
    static var lengthInBytes: Int { T.lengthInBytes * 4 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        destination[offset] = r
        offset += 1
        destination[offset] = g
        offset += 1
        destination[offset] = b
        offset += 1
        destination[offset] = a
        offset += 1
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (RGBA<T>, Int) {
        var offset = offset
        let r = source[offset]
        offset += 1
        let g = source[offset]
        offset += 1
        let b = source[offset]
        offset += 1
        let a = source[offset]
        offset += 1
        return (RGBA(r: r, g: g, b: b, a: a), offset)
    }
}
