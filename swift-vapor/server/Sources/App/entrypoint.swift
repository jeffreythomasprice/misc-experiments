import Dispatch
import Logging
import Vapor

class PendingClient {
    let id: UUID
    var name: String

    init(name: String) {
        id = UUID()
        self.name = name
    }

    func description() -> String {
        return "PendingClient(id=\(id), name=\(name))"
    }
}

class ConnectedClient {
    let ws: WebSocket
    let id: UUID
    var name: String

    init(ws: WebSocket, client: PendingClient) {
        self.ws = ws
        self.id = client.id
        self.name = client.name
    }

    func description() -> String {
        return "ConnectedClient(id=\(id), name=\(name))"
    }

    func send(message: ServerToClientMessage) throws {
        ws.send(try String(decoding: JSONEncoder().encode(message), as: UTF8.self))
    }
}

enum ConnectedClientServiceError: Error {
    case noSuchClient
}

class ConnectedClientsService {
    private let logger: Logger = Logger(label: "clients")

    private var pendingClients: [UUID: PendingClient] = [:]
    private var connectedClients: [UUID: ConnectedClient] = [:]

    func close() async {
        logger.debug("closing all active clients")
        for client in connectedClients.values {
            await close(client: client)
        }
        pendingClients.removeAll()
        connectedClients.removeAll()
    }

    func close(client: ConnectedClient) async {
        logger.debug("closing client \(client.description())")
        do {
            try await client.ws.close()
        } catch {
            logger.error("error closing client websocket \(error)")
        }
        connectedClients.removeValue(forKey: client.id)
    }

    func newClient(name: String) -> PendingClient {
        let result = PendingClient(name: name)
        logger.debug("new pending client, name = \(name), id = \(result.id)")
        pendingClients[result.id] = result
        logger.trace("there are now \(pendingClients.count) pending clients")
        return result
    }

    func upgradeClient(id: UUID, ws: WebSocket) throws -> ConnectedClient {
        if let client = pendingClients.removeValue(forKey: id) {
            let result = ConnectedClient(ws: ws, client: client)
            connectedClients[client.id] = result
            return result
        } else {
            throw ConnectedClientServiceError.noSuchClient
        }
    }

    func broadcast(sender: ConnectedClient, message: String) {
        logger.trace("sending message from \(sender.description()), message = \(message)")
        let messageObj = ServerToClientMessage.send(
            ServerToClientMessage.Send(senderId: sender.id.uuidString, message: message))
        for (_, client) in connectedClients {
            if client.id != sender.id {
                do {
                    try client.send(message: messageObj)
                } catch {
                    logger.error("error sending to \(client.description()): \(error)")
                }
            }
        }
    }
}

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
                    // ignore whatever the env var said and always print trace
                    level: .trace
                )
            }
        }

        let app = Application(env)
        defer { app.shutdown() }

        app.http.server.configuration.hostname = "127.0.0.1"
        app.http.server.configuration.port = 8001

        do {
            let connectedClients = ConnectedClientsService()
            try await configure(app, connectedClients: connectedClients)
        } catch {
            app.logger.report(error: error)
            throw error
        }
        try await app.runFromAsyncMainEntrypoint()
    }
}

private func configure(
    _ app: Application,
    connectedClients: ConnectedClientsService
) async throws {
    // TODO handle ctrl+c faster, currently takes like 10 seconds to timeout shutting down

    let corsConfiguration = CORSMiddleware.Configuration(
        allowedOrigin: .all,
        allowedMethods: [.GET, .POST, .PUT, .OPTIONS, .DELETE, .PATCH],
        allowedHeaders: [.contentType]
    )
    let cors = CORSMiddleware(configuration: corsConfiguration)
    app.middleware.use(cors, at: .beginning)

    app.post("login") { req -> LoginResponse in
        let request = try req.content.decode(LoginRequest.self)
        req.logger.debug("login request = \(request)")
        let client = connectedClients.newClient(name: request.name)
        let response = LoginResponse(id: client.id.uuidString)
        req.logger.debug("login response = \(response)")
        return response
    }

    app.webSocket("ws") { req, ws in
        req.logger.debug(
            "new ws connection \(req.remoteAddress?.description ?? "no remote address")")

        class State {
            var client: ConnectedClient?

            init() {
                client = nil
            }
        }

        let state = State()

        ws.onClose.whenComplete { _ in
            if let client = state.client {
                Task {
                    await connectedClients.close(client: client)
                }
            }
        }

        ws.onText { ws, messageJson in
            let message: ClientToServerMessage
            do {
                message = try JSONDecoder().decode(
                    ClientToServerMessage.self, from: messageJson.data(using: .utf8)!)
            } catch {
                req.logger.warning("\(error)")
                return
            }

            switch message {
            case .send(let send):
                if let client = state.client {
                    connectedClients.broadcast(sender: client, message: send.message)
                } else {
                    req.logger.warning("not logged in, can't send messages")
                    _ = ws.close(code: .normalClosure)
                }

            case .login(let login):
                if let client = state.client {
                    req.logger.warning("already logged in, can't handle request \(login)")
                    Task {
                        await connectedClients.close(client: client)
                    }
                    return
                }

                req.logger.info("logged in \(login)")
                if let id = UUID.init(uuidString: login.id) {
                    do {
                        state.client = try connectedClients.upgradeClient(id: id, ws: ws)
                    } catch {
                        req.logger.error("error upgrading client \(error)")
                    }
                } else {
                    req.logger.warning("client gave us nonsense for an id \(login)")
                }
            }
        }
    }
}
