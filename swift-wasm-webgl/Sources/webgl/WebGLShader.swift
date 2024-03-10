import JavaScriptKit

class WebGLShader {
    enum Error: Swift.Error {
        case compile(String)
        case link(String)
    }

    struct AttributeInfo {
        let index: Int
        let size: Int
        let type: Int
        let name: String
    }

    struct UniformInfo {
        let location: JSValue
        let size: Int
        let type: Int
        let name: String
    }

    private let gl: JSValue
    private let vertexShader: JSValue
    private let fragmentShader: JSValue
    private let program: JSValue

    let attributes: [String: AttributeInfo]
    let uniforms: [String: UniformInfo]

    init(gl: JSValue, vertexSource: String, fragmentSource: String) throws {
        self.gl = gl

        vertexShader = try WebGLShader.createShader(gl: gl, type: gl.VERTEX_SHADER, source: vertexSource).get()

        fragmentShader =
            switch WebGLShader.createShader(gl: gl, type: gl.FRAGMENT_SHADER, source: fragmentSource) {
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
            throw Error.link(log)
        }

        let activeAttributes = Int(gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES).number!)
        var attributes = [String: AttributeInfo]()
        for i in 0..<activeAttributes {
            let info = gl.getActiveAttrib(program, i)
            let name = info.name.string!
            attributes[name] = AttributeInfo(
                index: i,
                size: Int(info.size.number!),
                type: Int(info.type.number!),
                name: name
            )
        }
        self.attributes = attributes

        let activeUniforms = Int(gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS).number!)
        var uniforms = [String: UniformInfo]()
        for i in 0..<activeUniforms {
            let info = gl.getActiveUniform(program, i)
            let name = info.name.string!
            uniforms[name] = UniformInfo(
                location: gl.getUniformLocation(program, name),
                size: Int(info.size.number!),
                type: Int(info.type.number!),
                name: name
            )
        }
        self.uniforms = uniforms
    }

    // TODO dispose?

    func use() {
        _ = gl.useProgram(program)
    }

    private static func createShader(gl: JSValue, type: JSValue, source: String) -> Result<JSValue, Error> {
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
