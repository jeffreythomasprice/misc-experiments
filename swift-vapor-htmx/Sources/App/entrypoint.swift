import Dispatch
import Leaf
import Logging
import Vapor

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

    do {
      try await configure(app)
    } catch {
      app.logger.report(error: error)
      throw error
    }
    try await app.runFromAsyncMainEntrypoint()
  }
}

public func configure(_ app: Application) async throws {
  app.http.server.configuration.port = 8000

  // uncomment to serve files from /Public folder
  // app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

  app.views.use(.leaf)

  try routes(app)
}

func routes(_ app: Application) throws {
  app.get { req async throws in
    try await req.view.render("index", [String: String]())
  }

  var count = 0
  let clickResults = { (req: Request) async throws -> View in
    try await req.view.render("clickResults", ["count": count.description])
  }

  app.get("click") { req async throws in
    try await clickResults(req)
  }

  app.post("click") { req async throws -> View in
    count += 1
    return try await clickResults(req)
  }
}
