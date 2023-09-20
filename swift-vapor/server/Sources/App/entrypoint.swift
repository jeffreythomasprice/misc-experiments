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
        try LoggingSystem.bootstrap(from: &env) { level in
            let console = Terminal()
            return { (label: String) in
                return ConsoleLogger(
                    label: label,
                    console: console,
                    // ignore whatever the env var said and just always print trace
                    level: .trace
                )
            }
        }

        let app = Application(env)
        defer { app.shutdown() }

        app.http.server.configuration.hostname = "127.0.0.1"
        app.http.server.configuration.port = 8001

        do {
            try await configure(app)
        } catch {
            app.logger.report(error: error)
            throw error
        }
        try await app.runFromAsyncMainEntrypoint()
    }
}

struct LoginRequest: Content {
    var name: String
}

struct LoginResponse: Content {
    var id: String
}

struct ClientToServerMessage: Content {
    var message: String
}

struct ServerToClientMessage: Content {
    var senderId: String
    var message: String
}

private func configure(_ app: Application) async throws {
    // app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

    let corsConfiguration = CORSMiddleware.Configuration(
        allowedOrigin: .all,
        allowedMethods: [.GET, .POST, .PUT, .OPTIONS, .DELETE, .PATCH],
        allowedHeaders: [.contentType]
    )
    let cors = CORSMiddleware(configuration: corsConfiguration)
    // cors middleware should come before default error middleware using `at: .beginning`
    app.middleware.use(cors, at: .beginning)

    // app.get { req async in
    //     req.fileio.streamFile(at: "Public/index.html")
    // }

    app.post("login") { req -> LoginResponse in
        let request = try req.content.decode(LoginRequest.self)
        req.logger.debug("login request = \(request)")
        let response = LoginResponse(id: UUID().uuidString)
        req.logger.debug("login response = \(response)")
        return response
    }

    app.webSocket("ws") { req, ws in
        ws.onText { ws, messageJson in
            guard
                let message = try? JSONDecoder().decode(
                    ClientToServerMessage.self, from: messageJson.data(using: .utf8)!)
            else {
                return
            }
            req.logger.debug("received ws message = \(message)")
            let response = ServerToClientMessage(
                senderId: "TODO fill in sender ID",
                message: "response from server"
            )
            req.logger.debug("responding to ws message = \(response)")
            guard
                let response = try? String(decoding: JSONEncoder().encode(response), as: UTF8.self)
            else {
                return
            }
            ws.send(response)
        }
    }
}
