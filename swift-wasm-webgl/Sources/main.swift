import JavaScriptEventLoop
import JavaScriptKit

enum ShaderError: Error {
    case compile(String)
    case link(String)
}

class Shader {
    private let gl: JSValue
    private let vertexShader: JSValue
    private let fragmentShader: JSValue
    private let program: JSValue

    init(gl: JSValue, vertexSource: String, fragmentSource: String) throws {
        self.gl = gl

        vertexShader = try Shader.createShader(gl: gl, type: gl.VERTEX_SHADER, source: vertexSource).get()

        fragmentShader =
            switch Shader.createShader(gl: gl, type: gl.FRAGMENT_SHADER, source: fragmentSource) {
            case let .success(x):
                x
            case let .failure(e):
                _ = gl.deleteShader(vertexShader)
                throw e
            }

        program = gl.createProgram()
        _ = gl.attachShader(program, vertexShader)
        _ = gl.attachShader(program, fragmentShader)
        _ = gl.linkProgram(program)
        if !self.gl.getProgramParameter(program, gl.LINK_STATUS).boolean! {
            let log = gl.getProgramInfoLog(program).string!
            _ = self.gl.deleteShader(vertexShader)
            _ = self.gl.deleteShader(fragmentShader)
            _ = self.gl.deleteProgram(program)
            throw ShaderError.link(log)
        }
    }

    private static func createShader(gl: JSValue, type: JSValue, source: String) -> Result<JSValue, ShaderError> {
        let result = gl.createShader(type)
        _ = gl.shaderSource(result, source)
        _ = gl.compileShader(result)
        if !gl.getShaderParameter(result, gl.COMPILE_STATUS).boolean! {
            let log = gl.getShaderInfoLog(result).string!
            _ = gl.deleteShader(result)
            return .failure(.compile(log))
        }
        return .success(result)
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

do {
    let shader = try Shader(
        gl: gl,
        vertexSource: """
            attribute vec2 positionAttribute;

            void main() {
                gl_Position = vec4(positionAttribute, 0, 1);
            }
            """,
        fragmentSource: """
            precision mediump float;

            void main() {
                gl_FragColor = vec4(1, 1, 1, 1);
            }
            """
    )
} catch {
    print("error making shader: \(error)")
}

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

    _ = JSObject.global.window.requestAnimationFrame(animate)
    return .undefined
}
_ = JSObject.global.window.requestAnimationFrame(animate)
