import Dispatch
import Logging
import Vapor

/// This extension is temporary and can be removed once Vapor gets this support.
extension Vapor.Application {
    fileprivate static let baseExecutionQueue = DispatchQueue(label: "vapor.codes.entrypoint")

    fileprivate func runFromAsyncMainEntrypoint() async throws {
        try await withCheckedThrowingContinuation { continuation in
            Vapor.Application.baseExecutionQueue.async { [self] in
                do {
                    try self.run()
                    continuation.resume()
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}

@main
enum Entrypoint {
    static func main() async throws {
        var env = try Environment.detect()
        try LoggingSystem.bootstrap(from: &env)

        let app = Application(env)
        defer { app.shutdown() }

        app.http.server.configuration.hostname = "127.0.0.1"
        app.http.server.configuration.port = 8000

        do {
            try await configure(app)
        } catch {
            app.logger.report(error: error)
            throw error
        }
        try await app.runFromAsyncMainEntrypoint()
    }
}

struct MessageRequest: Content {
    var message: String
}

struct MessageResponse: Content {
    var message: String
}

private func configure(_ app: Application) async throws {
    app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

    app.get { req async in
        req.fileio.streamFile(at: "Public/index.html")
    }

    app.get("hello") { req async -> String in
        "Hello, world!"
    }

    app.post("test") { req -> MessageResponse in
        let request = try req.content.decode(MessageRequest.self)
        req.logger.debug("message request = \(request)")
        let response = MessageResponse(message: "response from server!")
        req.logger.debug("message response = \(response)")
        return response
    }

    app.webSocket("ws") { req, ws in
        ws.onText { ws, message in
            req.logger.debug("received ws message = \(message)")
            let response = "response from server"
            req.logger.debug("responding to ws message = \(response)")
            ws.send(response)
        }
    }
}
