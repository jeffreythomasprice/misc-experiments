import JavaScriptEventLoop
import JavaScriptKit

struct Vertex {
    let position: Vector2<Float32>
    let color: RGBA<Float32>
}

extension Vertex: TypedArraySerialization {
    typealias T = Float32

    func WriteTo(destination: JavaScriptKit.JSTypedArray<Float32>, offset: Int) -> Int {
        var offset = offset
        offset = position.WriteTo(destination: destination, offset: offset)
        offset = color.WriteTo(destination: destination, offset: offset)
        return offset
    }

    static func ReadFrom(source: JavaScriptKit.JSTypedArray<Float32>, offset: Int) -> (Vertex, Int) {
        var offset = offset
        let position: Vector2<Float32>
        (position, offset) = Vector2<Float32>.ReadFrom(source: source, offset: offset)
        let color: RGBA<Float32>
        (color, offset) = RGBA<Float32>.ReadFrom(source: source, offset: offset)
        return (Vertex(position: position, color: color), offset)
    }
}

JavaScriptEventLoop.installGlobalExecutor()

var canvas = JSObject.global.document.createElement("canvas")
canvas.style = .string(
    [
        "position": "absolute",
        "left": "0px",
        "top": "0px",
        "width": "100%",
        "height": "100%",
    ]
    .map { key, value in "\(key): \(value)" }
    .joined(separator: "; "))
_ = JSObject.global.document.body.replaceChildren(canvas)

var gl = canvas.getContext("webgl2", ["powerPreference": "high-performance"])

let shader: Shader
do {
    shader = try Shader(
        gl: gl,
        vertexSource: """
            attribute vec2 positionAttribute;
            attribute vec4 colorAttribute;

            varying vec4 colorVarying;

            void main() {
                gl_Position = vec4(positionAttribute, 0, 1);
                colorVarying = colorAttribute;
            }
            """,
        fragmentSource: """
            precision mediump float;

            varying vec4 colorVarying;

            void main() {
                gl_FragColor = colorVarying;
            }
            """
    )
} catch {
    print("error making shader: \(error)")
    exit(0)
}

let arrayBufferData = JSTypedArray<Float32>(length: 6 * 4)
_ = [
    Vertex(
        position: Vector2(x: -0.5, y: 0.5),
        color: RGBA(r: 1, g: 1, b: 0, a: 1)
    ),
    Vertex(
        position: Vector2(x: 0.5, y: 0.5),
        color: RGBA(r: 0, g: 1, b: 1, a: 1)
    ),
    Vertex(
        position: Vector2(x: 0.5, y: -0.5),
        color: RGBA(r: 1, g: 0, b: 1, a: 1)
    ),
    Vertex(
        position: Vector2(x: -0.5, y: -0.5),
        color: RGBA(r: 0.5, g: 0, b: 1, a: 1)
    ),
]
.WriteTo(destination: arrayBufferData, offset: 0)

let arrayBuffer = gl.createBuffer()
_ = gl.bindBuffer(gl.ARRAY_BUFFER, arrayBuffer)
_ = gl.bufferData(gl.ARRAY_BUFFER, arrayBufferData, gl.STATIC_DRAW)
_ = gl.bindBuffer(gl.ARRAY_BUFFER, JSValue.null)

let elementArrayBuffer = gl.createBuffer()
_ = gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, elementArrayBuffer)
_ = gl.bufferData(
    gl.ELEMENT_ARRAY_BUFFER,
    JSTypedArray<UInt16>([
        0, 1, 2,
        2, 3, 0,
    ]),
    gl.STATIC_DRAW
)
_ = gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, JSValue.null)

func resize() {
    let width = Int(JSObject.global.window.innerWidth.number!)
    let height = Int(JSObject.global.window.innerHeight.number!)
    canvas.width = .number(Double(width))
    canvas.height = .number(Double(height))

    _ = gl.viewport(0, 0, width, height)
}
_ = JSObject.global.window.addEventListener(
    "resize",
    JSClosure { _ in
        resize()
        return .undefined
    })
resize()

let animate = JSClosure { time in
    _ = gl.clearColor(0.25, 0.5, 0.75, 1.0)
    _ = gl.clear(gl.COLOR_BUFFER_BIT)

    shader.use()

    // TODO use vertex array

    _ = gl.bindBuffer(gl.ARRAY_BUFFER, arrayBuffer)
    _ = gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, elementArrayBuffer)

    let positionAttribute = shader.attributes["positionAttribute"]!
    let colorAttribute = shader.attributes["colorAttribute"]!
    _ = gl.enableVertexAttribArray(positionAttribute.index)
    _ = gl.enableVertexAttribArray(colorAttribute.index)
    _ = gl.vertexAttribPointer(positionAttribute.index, 2, gl.FLOAT, false, 4 * 6, 0)
    _ = gl.vertexAttribPointer(colorAttribute.index, 4, gl.FLOAT, false, 4 * 6, 4 * 2)

    _ = gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0)

    _ = gl.disableVertexAttribArray(positionAttribute.index)
    _ = gl.disableVertexAttribArray(colorAttribute.index)

    _ = gl.bindBuffer(gl.ARRAY_BUFFER, JSValue.null)
    _ = gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, JSValue.null)

    _ = gl.useProgram(JSValue.null)

    _ = JSObject.global.window.requestAnimationFrame(animate)
    return .undefined
}
_ = JSObject.global.window.requestAnimationFrame(animate)
