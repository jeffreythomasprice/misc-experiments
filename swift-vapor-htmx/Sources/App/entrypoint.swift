import Dispatch
import JWT
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

struct AuthMiddleware: AsyncMiddleware {
	func respond(to request: Vapor.Request, chainingTo next: Vapor.AsyncResponder) async throws -> Vapor.Response {
		if try await isAuthenticated(request: request) {
			return try await next.respond(to: request)
		} else {
			return request.redirect(to: "/login")
		}
	}

	func isAuthenticated(request: Vapor.Request) async throws -> Bool {
		let log = request.application.logger
		let auth = request.cookies["auth"]?.string
		log.debug("auth cookie = \(auth.debugDescription)")
		switch auth {
		case .some(_):
			// TODO validate cookie here
			return true
		case .none:
			return false
		}
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

	// TODO testing
	app.jwt.signers.use(.hs256(key: "TODO should be a random string"))
	struct TestJWTPayload: JWTPayload {
		let username: String
		let exp: ExpirationClaim

		func verify(using signer: JWTSigner) throws {
			try self.exp.verifyNotExpired()
		}
	}
	let expirationDate = Date.now.advanced(by: 60)
	let jwt = try app.jwt.signers.sign(
		TestJWTPayload(
			username: "foobar",
			exp: ExpirationClaim(
				value: Date(timeIntervalSince1970: TimeInterval(UInt64(expirationDate.timeIntervalSince1970)))))
	)
	log.debug("TODO jwt = \(jwt)")
	let decodedJwt: TestJWTPayload = try app.jwt.signers.verify(jwt)
	log.debug("TODO decoded = \(decodedJwt)")

	try routes(app: app, db: db)
}

func routes(app: Application, db: DBService) throws {
	let log = app.logger

	let authMiddleware = AuthMiddleware()

	app.get { req async throws -> Vapor.Response in
		if try await authMiddleware.isAuthenticated(request: req) {
			req.redirect(to: "/loggedIn")
		} else {
			req.redirect(to: "/login")
		}
	}

	app.get("login") { req async throws -> Vapor.View in
		try await req.view.render("login")
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

	app.group(authMiddleware) {
		$0.get("loggedIn") { req async throws -> Vapor.View in
			try await req.view.render("loggedIn")
		}
	}
}
