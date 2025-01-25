import Foundation
import Hummingbird
import HummingbirdRouter
import Logging
import SwiftDotenv

LoggingSystem.bootstrap { name in
	PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout()))
}

var logger = Logger(label: "Experiment")
logger.logLevel = .trace

try Dotenv.configure(atPath: "local.env")

let env = Env(logger: logger)

// TODO db stuff
// let db = await Database(logger: logger.child(logLevel: .info, label: "DB"), env: env)

actor Counter {
	var count = 0

	func increment(by: Int) -> Int {
		count += by
		return count
	}
}

let counter = Counter()

struct Json<T>: ResponseGenerator where T: Codable {
	let value: T

	init(_ value: T) {
		self.value = value
	}

	func response(from request: HummingbirdCore.Request, context: some Hummingbird.RequestContext)
		throws -> HummingbirdCore.Response
	{
		// TODO 415 if utf8 isn't allowed, or json isn't allowed
		let data = try newJSONEncoder().encode(value)
		if let s = String(data: data, encoding: .utf8) {
			var response = s.response(from: request, context: context)
			response.headers[.contentType] =
				MediaType.applicationJson.withParameter(name: "charset", value: "utf8").description
			return response
		} else {
			throw HTTPError(.internalServerError)
		}
	}
}

let router = RouterBuilder(context: ExtendedRequestContext.self) {
	CORSMiddleware()
	LogRequestsMiddleware(.trace, includeHeaders: .all())

	FileMiddleware(
		"../client/static", urlBasePath: "/", cacheControl: .init([]), searchForIndexHtml: true,
		logger: logger.child(label: "FilesMiddleware")
	)
	FileMiddleware(
		"../client/dist", urlBasePath: "/", cacheControl: .init([]), searchForIndexHtml: true,
		logger: logger.child(label: "FilesMiddleware")
	)

	RouteGroup("/count") {
		Get { request, context async throws -> Json<ExampleResponse> in
			// try await ExampleResponse(count: counter.count).jsonString()!
			await Json(ExampleResponse(count: counter.count))
		}
		Post { request, context async throws -> Json<ExampleResponse> in
			let requestBody = try await request.decode(
				as: ExampleRequest.self, context: context
			)
			let newCount = await counter.increment(by: requestBody.incrementBy)
			return Json(ExampleResponse(count: newCount))
		}
	}
}

var app = Application(
	router: router,
	configuration: .init(address: .hostname(env.assert("HOST"), port: env.assertInt("PORT"))),
	logger: logger.child(label: "App")
)

// TODO db stuff
// app.addServices(await db.client)

// app.beforeServerStarts {
//     try await db.migrate()
// }

do {
	try await app.runService()
} catch {
	logger.critical("server error: \(String(reflecting: error))")
}
