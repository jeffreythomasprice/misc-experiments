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
            let connectedClients = ConnectedClientsService()
            try await configure(app, connectedClients: connectedClients)
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

class PendingClient {
    let id: UUID
    var name: String

    init(name: String) {
        id = UUID()
        self.name = name
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

    func send(message: ServerToClientMessage) {
        // TODO JEFF implement send

        /*
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
        */
    }
}

enum ConnectedClientServiceError: Error {
    case noSuchClient
}

class ConnectedClientsService {
    private let logger: Logger = Logger(label: "clients")

    private var pendingClients: [UUID: PendingClient] = [:]
    private var connectedClients: [UUID: ConnectedClient] = [:]

    func close() {
        logger.debug("closing all active clients")
        // TODO JEFF disconnect everybody
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
        logger.trace("sending message from \(sender), message = \(message)")
        let messageObj = ServerToClientMessage(senderId: sender.id.uuidString, message: message)
        for (_, client) in connectedClients {
            if client.id != sender.id {
                client.send(message: messageObj)
            }
        }
    }
}

private func configure(
    _ app: Application,
    connectedClients: ConnectedClientsService
) async throws {
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
        let client = connectedClients.newClient(name: request.name)
        let response = LoginResponse(id: client.id.uuidString)
        req.logger.debug("login response = \(response)")
        return response
    }

    app.webSocket("ws") { req, ws in
        req.logger.debug("TODO JEFF new ws connection \(req.remoteAddress)")

        ws.onClose.whenComplete { _ in
            connectedClients.close()
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

            req.logger.debug("TODO JEFF message = \(message)")

            // TODO JEFF broadcast
            // connectedClients.broadcast(sender: ConnectedClient, message: String)
        }
    }
}
