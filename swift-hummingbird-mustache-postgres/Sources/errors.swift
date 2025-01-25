import Elementary
import Hummingbird
import HummingbirdElementary

private class Content: HTML {
    private let message: String

    init(message: String) {
        self.message = message
    }

    var content: some HTML {
        ErrorMessages(messages: [message])
    }
}

extension HTTPError {
    fileprivate var message: String {
        self.status.reasonPhrase
    }
}

private func responseForError(request: Request, context: ExtendedRequestContext, error: HTTPError) -> Response {
    do {
        var response = try HTMLResponse {
            IndexPage(context: context, content: Content(message: error.message))
        }.response(from: request, context: context)
        response.status = error.status
        return response
    } catch {
        context.logger.error("failed to render response from previous error, new error: \(String(reflecting: error))")
        return Response(status: .internalServerError)
    }
}

struct ErrorMiddleware<Context: ExtendedRequestContext>: RouterMiddleware {
    func handle(_ request: Request, context: Context, next: (Request, Context) async throws -> Response) async throws -> Response {
        var logger = context.logger
        logger[metadataKey: "method"] = "\(request.method)"
        logger[metadataKey: "uri"] = "\(request.uri)"
        do {
            return try await next(request, context)
        } catch let error as HTTPError {
            context.logger.warning("handler failed http error: \(String(reflecting: error))")
            return responseForError(request: request, context: context, error: error)
        } catch {
            context.logger.warning("handler failed other error: \(String(reflecting: error))")
            return responseForError(request: request, context: context, error: HTTPError(.internalServerError))
        }
    }
}
