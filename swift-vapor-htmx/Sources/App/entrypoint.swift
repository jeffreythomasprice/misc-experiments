import Dispatch
import Leaf
import Logging
import SQLite
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
		LoggingSystem.bootstrap(console: Terminal(), level: .debug, metadata: [String: Logger.MetadataValue]())

		let env = try Environment.detect()
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
	let log = app.logger

	app.http.server.configuration.port = 8000

	// uncomment to serve files from /Public folder
	// app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

	app.views.use(.leaf)

	let db = try DBService(
		log: log,
		connection: try Connection(.uri("./db.sqlite"))
	)

	try routes(app: app, db: db)
}

func routes(app: Application, db: DBService) throws {
	let log = app.logger

	app.get { req async throws -> Vapor.Response in
		return switch req.cookies["auth"]?.string {
		// TODO actually check if auth is a valid jwt
		case .some(_):
			req.redirect(to: "/loggedIn")
		case .none:
			req.redirect(to: "/login")
		}
	}

	app.get("login") { req async throws -> Vapor.View in
		try await req.view.render("login")
	}

	app.get("loggedIn") { req async throws -> Vapor.View in
		try await req.view.render("loggedIn")
	}

	app.post("api", "login") { req async throws -> Vapor.Response in
		struct Request: Content {
			let username: String
			let password: String
		}
		let reqBody = try req.content.decode(Request.self)
		log.debug("login: \(reqBody)")

		switch db.getUser(username: reqBody.username, password: reqBody.password)
		{
		case .success(.some(_)):
			let res = Response()

			var cookie = HTTPCookies.Value(string: "TODO jwt here")
			// TODO use jwt expiration, in seconds
			cookie.expires = Date.now.advanced(by: 60 * 5)
			res.headers.setCookie = ["auth": cookie]

			res.headers.add(name: "hx-location", value: "/loggedIn")

			return res
		// TODO use a struct for errors?
		case .success(.none):

			return try await req.view.render(
				"loginError",
				[
					"message": "Invalid credentials"
				]
			).encodeResponse(for: req)
		case .failure(let e):
			return try await req.view.render(
				"loginError",
				[
					"message": "Unexpected error: \(e)"
				]
			).encodeResponse(for: req)
		}
	}
}
