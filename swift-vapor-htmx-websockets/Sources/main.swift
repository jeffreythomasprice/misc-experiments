import Leaf
import Vapor

struct WebsocketInput: Codable {
    let message: String
}

struct WebsocketOutput: Codable {
    let message: String
}

LoggingSystem.bootstrap { label in
    var result = StreamLogHandler.standardOutput(label: label)
    result.logLevel = .trace
    return result
}

let app = Application()
defer { app.shutdown() }

app.http.server.configuration.port = 8000

app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

app.views.use(.leaf)

app.get { req in
    return req.view.render("index")
}

app.webSocket("ws") { req, ws in
    let clientDescription = "websocket \(req.remoteAddress?.description ?? "<no remote addr>")"
    let logger = Logger(label: clientDescription)
    logger.debug("new connection")

    ws.onText { ws, text in
        logger.debug("received text message, raw: \(text)")
        do {
            let message = try JSONDecoder().decode(WebsocketInput.self, from: ByteBuffer(string: text))
            logger.info("received text message: \(message.message)")

            var response = try await app.leaf.renderer.render(path: "websocket-message", context: WebsocketOutput(message: message.message))
                .get()
            if let responseStr = response.readString(length: response.readableBytes) {
                logger.debug("response: \(responseStr)")
                try await ws.send(responseStr)
            } else {
                logger.error("failed to render a valid response string")
            }
        } catch {
            logger.error("error handling text message: \(error)")
        }
    }

    // TODO binary?
    // ws.onBinary { ws, data in
    //     logger.info("received binary message: \(data.readableBytes) bytes")
    // }

    // do {
    //     try! await ws.close()
    // } catch {
    //     logger.error("error closing: \(error)")
    // }
}

try app.run()
