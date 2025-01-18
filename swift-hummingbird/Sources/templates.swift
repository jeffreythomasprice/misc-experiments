import Hummingbird
import Logging
import Mustache

enum TemplateErrors: Error {
    case utf8Error(template: String)
    case renderError(template: String)
}

extension TemplateErrors: HasErrorData {
    var errorData: ErrorData {
        ErrorData(message: "\(self)")
    }
}

final class Templates: Sendable {
    private let logger: Logger
    private let mustacheLibrary: MustacheLibrary

    init(logger: Logger, directory: String, withExtension: String) async throws {
        self.logger = logger
        self.mustacheLibrary = try await MustacheLibrary.init(
            directory: directory,
            withExtension: withExtension
        )
    }

    func renderToString(_ object: Any, withTemplate template: String) throws -> String {
        if case let .some(result) = mustacheLibrary.render(object, withTemplate: template) {
            return result
        } else {
            throw TemplateErrors.renderError(template: template)
        }
    }

    func renderToByteBuffer(_ object: Any, withTemplate template: String) throws -> ByteBuffer {
        let result = try renderToString(object, withTemplate: template)
        if case let .some(data) = result.data(using: .utf8) {
            return .init(bytes: data)
        } else {
            throw TemplateErrors.utf8Error(template: template)
        }
    }

    func renderToResponse(_ object: Any, withTemplate template: String) throws -> Response {
        let byteBuffer = try renderToByteBuffer(object, withTemplate: template)
        return Response(
            status: .ok, headers: [.contentType: "\(contentType(name: template)); charset=utf-8"],
            body: .init(byteBuffer: byteBuffer))
    }

    private func contentType(name: String) -> String {
        let ext = name.pathExtension
        return
            switch ext
        {
        case "html":
            "text/html"
        case "txt":
            "text/plain"
        default:
            {
                logger.warning("unrecognized file extension \(ext), assuming text")
                return "text/plain"
            }()
        }
    }
}
