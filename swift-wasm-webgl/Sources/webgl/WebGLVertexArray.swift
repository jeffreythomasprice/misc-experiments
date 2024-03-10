import JavaScriptKit

class WebGLVertexArray<T: TypedArraySerialization & StaticSized> {
    enum VertexType {
        case byte
        case short
        case unsignedByte
        case unsignedShort
        case float
        case halfFloat
        case int
        case unsignedInt
        case int_2_10_10_10_rev
        case unsignedInt_2_10_10_10_rev

        func glValue(gl: JSValue) -> Int {
            switch self {
            case .byte:
                return Int(gl.BYTE.number!)
            case .short:
                return Int(gl.SHORT.number!)
            case .unsignedByte:
                return Int(gl.UNSIGNED_BYTE.number!)
            case .unsignedShort:
                return Int(gl.UNSIGNED_SHORT.number!)
            case .float:
                return Int(gl.FLOAT.number!)
            case .halfFloat:
                return Int(gl.HALF_FLOAT.number!)
            case .int:
                return Int(gl.INT.number!)
            case .unsignedInt:
                return Int(gl.UNSIGNED_INT.number!)
            case .int_2_10_10_10_rev:
                return Int(gl.INT_2_10_10_10_REV.number!)
            case .unsignedInt_2_10_10_10_rev:
                return Int(gl.UNSIGNED_INT_2_10_10_10_REV.number!)
            }
        }
    }

    enum DrawType {
        case points
        case lineStrip
        case lineLoop
        case lines
        case triangleStrip
        case triangleFan
        case triangles

        func glValue(gl: JSValue) -> Int {
            switch self {
            case .points:
                return Int(gl.POINTS.number!)
            case .lineStrip:
                return Int(gl.LINE_STRIP.number!)
            case .lineLoop:
                return Int(gl.LINE_LOOP.number!)
            case .lines:
                return Int(gl.LINES.number!)
            case .triangleStrip:
                return Int(gl.TRIANGLE_STRIP.number!)
            case .triangleFan:
                return Int(gl.TRIANGLE_FAN.number!)
            case .triangles:
                return Int(gl.TRIANGLES.number!)
            }
        }
    }

    struct VertexAttributeInfo {
        let shaderInfo: WebGLShader.AttributeInfo
        let size: Int
        let type: VertexType
        let normalized: Bool
        let stride: Int
        let offset: Int
    }

    private let gl: JSValue
    private let vertexArray: JSValue

    init(
        gl: JSValue,
        shader: WebGLShader,
        arrayBuffer: WebGLBuffer<T>,
        elementArrayBuffer: WebGLBuffer<UInt16>,
        attributes: any Sequence<VertexAttributeInfo>
    ) {
        self.gl = gl
        vertexArray = gl.createVertexArray()
        _ = gl.bindVertexArray(vertexArray)
        shader.use()
        arrayBuffer.bind()
        elementArrayBuffer.bind()
        for attr in attributes {
            _ = gl.enableVertexAttribArray(attr.shaderInfo.index)
            _ = gl.vertexAttribPointer(
                attr.shaderInfo.index,
                attr.size,
                attr.type.glValue(gl: gl),
                attr.normalized,
                attr.stride,
                attr.offset
            )
        }
        _ = gl.bindVertexArray(JSValue.null)
    }

    // TODO dispose?

    func bind() {
        _ = gl.bindVertexArray(vertexArray)
    }

    func drawElements(drawType: DrawType, count: Int, offset: Int) {
        _ = gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0)
    }
}
