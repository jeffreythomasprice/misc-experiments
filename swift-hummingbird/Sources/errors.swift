import Hummingbird

protocol HasErrorData {
    var errorData: ErrorData { get }
}

class ErrorData: TemplateData {
    var currentUser: User?
    var navBar: NavBar?
    let message: String

    init(context: ExtendedRequestContext?, message: String) {
        (self.currentUser, _) = commonTemplateData(context: context)
        self.message = message
    }
}

struct ErrorMiddleware<Context: RequestContext>: RouterMiddleware {
    private let templates: Templates

    init(templates: Templates) {
        self.templates = templates
    }

    func handle(_ request: Request, context: Context, next: (Request, Context) async throws -> Response) async throws -> Response {
        var logger = context.logger
        logger[metadataKey: "method"] = "\(request.method)"
        logger[metadataKey: "uri"] = "\(request.uri)"
        do {
            return try await next(request, context)
        } catch let error as HasErrorData {
            context.logger.warning("handler failed with error template: \(String(reflecting: error))")
            let data = error.errorData
            do {
                return try templates.renderToResponse(data, withTemplate: "error.html")
            } catch {
                context.logger.error("failed to render template for \(data), new error: \(String(reflecting: error))")
                return Response(status: .internalServerError)
            }
        } catch let error as HTTPError {
            // TODO render a template for HTTP errors, special case for 404
            context.logger.warning("handler failed with http error: \(String(reflecting: error))")
            do {
                return try error.response(from: request, context: context)
            } catch {
                context.logger.error("failed to render response from previous http error, new error: \(String(reflecting: error))")
                return Response(status: .internalServerError)
            }
        } catch {
            context.logger.warning("handler failed other error: \(String(reflecting: error))")
            return Response(status: .internalServerError)
        }
    }
}
