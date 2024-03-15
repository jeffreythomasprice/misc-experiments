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

var camera = LookAtCamera<Float32>(
    position: Vector3(x: 0, y: 0, z: -6),
    target: Vector3(x: 0, y: 0, z: 0),
    up: Vector3(x: 0, y: 1, z: 0)
)

_ = JSObject.global.window.addEventListener(
    "mousemove",
    JSClosure { arg in
        let e = arg[0]

        let clientX = Int(e.clientX.number!)
        let clientY = Int(e.clientY.number!)

        let canvasWidth = Int(canvas.width.number!)
        let canvasHeight = Int(canvas.height.number!)

        let desiredX = canvasWidth / 2
        let desiredY = canvasHeight / 2

        // if we're grabbing the mouse, and we're not currently at the center of the canvas, do whatever this mouse movement indicates
        if JSObject.global.document.pointerLockElement == canvas && clientX != desiredX && clientY != desiredY {
            let movementX = Float32(e.movementX.number!)
            let movementY = Float32(e.movementY.number!)
            camera.turn(mouseMovement: Vector2(x: movementX, y: -movementY))
        }
        return .undefined
    })

_ = JSObject.global.window.addEventListener(
    "mouseup",
    JSClosure { arg in
        let e = arg[0]
        let button = Int(e.button.number!)
        if button == 0 {
            Task {
                do {
                    if JSObject.global.document.pointerLockElement == canvas {
                        _ = JSObject.global.document.exitPointerLock()
                    } else {
                        if let p = JSPromise(
                            from: canvas.requestPointerLock([
                                "unadjustedMovement": true
                            ]))
                        {
                            _ = try await p.value
                        }
                    }
                } catch {
                    print("mousemove error \(error)")
                }
            }
        }
        return .undefined
    })

var keyCodeState: [Int: Bool] = [:]

func getKeyCodeState(key: Int) -> Bool {
    keyCodeState[key] ?? false
}

func getKeyCodeState(keys: [Int]) -> Bool {
    keys
        .first { key in getKeyCodeState(key: key) }
        .map { _ in true }
        ?? false
}

_ = JSObject.global.window.addEventListener(
    "keydown",
    JSClosure { arg in
        let e = arg[0]
        let keyCode = Int(e.keyCode.number!)
        keyCodeState[keyCode] = true
        return .undefined
    })

_ = JSObject.global.window.addEventListener(
    "keyup",
    JSClosure { arg in
        let e = arg[0]
        let keyCode = Int(e.keyCode.number!)
        keyCodeState[keyCode] = false
        return .undefined
    })

var lastTime: Float64 = 0
let animate = JSClosure { args in
    let time = args[0].number!
    let timeDelta = Float32((time - lastTime) / 1000)
    lastTime = time
    // TODO Put math opers that take left or right hand side as the primitive type
    rotation = (rotation + Degrees<Float32>(90) * Degrees(timeDelta)).truncatingRemainder(dividingBy: Degrees(360))

    do {
        var forward: Float32 = 0
        if getKeyCodeState(keys: [
            // up
            38,
            // w
            87,
        ]) {
            forward += 1
        }
        if getKeyCodeState(keys: [
            // down
            40,
            // s
            83,
        ]) {
            forward -= 1
        }
        var strafe: Float32 = 0
        if getKeyCodeState(keys: [
            // right
            39,
            // d
            68,
        ]) {
            strafe -= 1
        }
        if getKeyCodeState(keys: [
            // left
            37,
            // a
            65,
        ]) {
            strafe += 1
        }
        var up: Float32 = 0
        if getKeyCodeState(keys: [
            // space
            32
        ]) {
            up += 1
        }
        if getKeyCodeState(keys: [
            // shift
            16
        ]) {
            up -= 1
        }
        let speed = timeDelta * 7
        camera.move(
            forward: forward * speed,
            strafe: strafe * speed,
            up: up * speed
        )
    }

    _ = gl.clearColor(0.25, 0.5, 0.75, 1.0)
    _ = gl.clear(gl.COLOR_BUFFER_BIT)

    _ = gl.uniformMatrix4fv(
        projectionMatrixUniform.location,
        true,
        Matrix4<Float32>.perspective(
            verticalFieldOfView: Degrees(45).radians,
            width: Float32(canvas.width.number!),
            height: Float32(canvas.height.number!),
            near: 0.1,
            far: 1000
        ).data
    )
    _ = gl.uniformMatrix4fv(
        modelViewMatrixUniform.location,
        false,
        // TODO also apply rotation
        camera.transformMatrix.data
    )

    vertexArray.bind()
    vertexArray.drawElements(drawType: .triangles, count: 6, offset: 0)

    _ = JSObject.global.window.requestAnimationFrame(animate)
    return .undefined
}
_ = JSObject.global.window.requestAnimationFrame(animate)
