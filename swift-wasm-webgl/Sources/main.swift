import JavaScriptEventLoop
import JavaScriptKit

struct Vertex {
    let position: Vector3<Float32>
    let color: RGBA<Float32>

    static var positionOffset: Int {
        0
    }

    static var colorOffset: Int {
        Vector3<Float32>.lengthInBytes
    }
}

extension Vertex: TypedArraySerialization & StaticSized {
    typealias T = Float32

    static var lengthInBytes: Int {
        Vector3<Float32>.lengthInBytes + RGBA<Float32>.lengthInBytes
    }

    func writeTo(destination: JavaScriptKit.JSTypedArray<Float32>, offset: Int) -> Int {
        var offset = offset
        offset = position.writeTo(destination: destination, offset: offset)
        offset = color.writeTo(destination: destination, offset: offset)
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<Float32>, offset: Int) -> (Vertex, Int) {
        var offset = offset
        let position: Vector3<Float32>
        (position, offset) = Vector3<Float32>.readFrom(source: source, offset: offset)
        let color: RGBA<Float32>
        (color, offset) = RGBA<Float32>.readFrom(source: source, offset: offset)
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

let shader: WebGLShader
do {
    shader = try WebGLShader(
        gl: gl,
        vertexSource: """
            attribute vec3 positionAttribute;
            attribute vec4 colorAttribute;

            uniform mat4 projectionMatrixUniform;
            uniform mat4 modelViewMatrixUniform;

            varying vec4 colorVarying;

            void main() {
                gl_Position = projectionMatrixUniform * modelViewMatrixUniform * vec4(positionAttribute, 1);
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

let positionAttribute = shader.attributes["positionAttribute"]!
let colorAttribute = shader.attributes["colorAttribute"]!

let projectionMatrixUniform = shader.uniforms["projectionMatrixUniform"]!
let modelViewMatrixUniform = shader.uniforms["modelViewMatrixUniform"]!

let arrayBuffer = WebGLBuffer<Vertex>(
    gl: gl,
    type: .array,
    usage: .staticDraw,
    collection: [
        Vertex(
            position: Vector3(x: -1, y: -1, z: 0),
            color: RGBA(r: 1, g: 1, b: 0, a: 1)
        ),
        Vertex(
            position: Vector3(x: 1, y: -1, z: 0),
            color: RGBA(r: 0, g: 1, b: 1, a: 1)
        ),
        Vertex(
            position: Vector3(x: 1, y: 1, z: 0),
            color: RGBA(r: 1, g: 0, b: 1, a: 1)
        ),
        Vertex(
            position: Vector3(x: -1, y: 1, z: 0),
            color: RGBA(r: 0.5, g: 0, b: 1, a: 1)
        ),
    ]
)

let elementArrayBuffer = WebGLBuffer<UInt16>(
    gl: gl,
    type: .elementArray,
    usage: .staticDraw,
    collection: [
        0, 1, 2,
        2, 3, 0,
    ]
)

let vertexArray = WebGLVertexArray(
    gl: gl,
    shader: shader,
    arrayBuffer: arrayBuffer,
    elementArrayBuffer: elementArrayBuffer,
    attributes: [
        WebGLVertexArray.VertexAttributeInfo(
            shaderInfo: positionAttribute,
            size: 3,
            type: .float,
            normalized: false,
            stride: Vertex.lengthInBytes,
            offset: Vertex.positionOffset
        ),
        WebGLVertexArray.VertexAttributeInfo(
            shaderInfo: colorAttribute,
            size: 4,
            type: .float,
            normalized: false,
            stride: Vertex.lengthInBytes,
            offset: Vertex.colorOffset
        ),
    ]
)

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

var rotation = Degrees<Float32>(0)
var lastTime: Float64 = 0
let animate = JSClosure { args in
    let time = args[0].number!
    let timeDelta = Float32((time - lastTime) / 1000)
    lastTime = time
    // TODO Put math opers that take left or right hand side as the primitive type
    rotation = (rotation + Degrees<Float32>(90) * Degrees(timeDelta)).truncatingRemainder(dividingBy: Degrees(360))

    _ = gl.clearColor(0.25, 0.5, 0.75, 1.0)
    _ = gl.clear(gl.COLOR_BUFFER_BIT)

    _ = gl.uniformMatrix4fv(
        projectionMatrixUniform.location,
        true,
        Matrix4<Float32>.perspective(
            verticalFieldOfView: Degrees(60).radians,
            width: Float32(canvas.width.number!),
            height: Float32(canvas.height.number!),
            near: 0.1,
            far: 1000
        ).data
    )
    _ = gl.uniformMatrix4fv(
        modelViewMatrixUniform.location,
        false,
        Matrix4<Float32>
            .identity
            .translate(
                Vector3(
                    x: 0,
                    y: 0,
                    z: -6
                )
            )
            .rotate(
                axis: Vector3(x: 0, y: 1, z: 0),
                angle: rotation.radians
            )
            .data
    )

    vertexArray.bind()
    vertexArray.drawElements(drawType: .triangles, count: 6, offset: 0)

    _ = JSObject.global.window.requestAnimationFrame(animate)
    return .undefined
}
_ = JSObject.global.window.requestAnimationFrame(animate)
