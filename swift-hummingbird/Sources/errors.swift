import Hummingbird

protocol HasErrorData {
    var errorData: ErrorData { get }
}

struct ErrorData {
    let message: String
}

struct ErrorMiddleware<Context: RequestContext>: RouterMiddleware {
    private let templates: Templates

    init(templates: Templates) {
        self.templates = templates
    }

    func handle(_ request: Request, context: Context, next: (Request, Context) async throws -> Response) async throws -> Response {
        do {
            return try await next(request, context)
        } catch let error as TemplateErrors {
            context.logger.warning("handler failed: \(error)")
            let data = error.errorData
            do {
                return try templates.renderToResponse(data, withTemplate: "error.html")
            } catch {
                context.logger.error("failed to render template for \(data), error: \(error)")
                return Response(status: .internalServerError)
            }
        } catch {
            context.logger.warning("handler failed: \(error)")
            return Response(status: .internalServerError)
        }
    }
}
