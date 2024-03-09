import JavaScriptKit

enum BufferUsage {
    case StaticDraw
    case DynamicDraw
    case StreamDraw
    case StaticRead
    case DynamicRead
    case StreamRead
    case StaticCopy
    case DynamicCopy
    case StreamCopy

    func glValue(gl: JSValue) -> Int {
        switch self {
        case .StaticDraw:
            return Int(gl.STATIC_DRAW.number!)
        case .DynamicDraw:
            return Int(gl.DYNAMIC_DRAW.number!)
        case .StreamDraw:
            return Int(gl.STREAM_DRAW.number!)
        case .StaticRead:
            return Int(gl.STATIC_READ.number!)
        case .DynamicRead:
            return Int(gl.DYNAMIC_READ.number!)
        case .StreamRead:
            return Int(gl.STREAM_READ.number!)
        case .StaticCopy:
            return Int(gl.STATIC_COPPY.number!)
        case .DynamicCopy:
            return Int(gl.DYNAMIC_COPY.number!)
        case .StreamCopy:
            return Int(gl.STREAM_COPY.number!)
        }
    }
}
