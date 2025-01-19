import FluentKit
import FluentPostgresDriver
import Foundation
import Hummingbird
import HummingbirdFluent
import HummingbirdRouter
import Logging
import Mustache
import SwiftDotenv
import Synchronization

LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

var logger = Logger(label: "Experiment")
logger.logLevel = .trace

try Dotenv.configure(atPath: "local.env")

let env = Env(logger: logger)

let fluent = await {
    var logger = logger
    logger.logLevel = .info
    return await initDb(logger: logger, env: env)
}()

let templates: Templates
do {
    templates = try await Templates.init(logger: logger, directory: "templates", withExtension: "mustache")
} catch {
    logger.critical("mustache init error: \(error)")
    exit(1)
}

func index(request: Request, context: any RequestContext, content: () async throws -> String) async throws -> Response {
    let content = try await content()
    return try templates.renderToResponse(["content": content], withTemplate: "index.html")
}

// TODO no
actor ClicksActor {
    var clicks = 0

    func inccrement() {
        clicks += 1
    }
}
let clicks = ClicksActor()

// TODO no
func clicksHandler(request: Request, context: any RequestContext) async throws -> Response {
    return try await index(request: request, context: context) {
        try templates.renderToString(["clicks": await clicks.clicks], withTemplate: "clicks.html")
    }
}

func loginPage(request: Request, context: MIMETypeAwareRequestContext) async throws -> Response {
    try await index(request: request, context: context) {
        try templates.renderToString([:], withTemplate: "login.html")
    }
}

let router = RouterBuilder(context: MIMETypeAwareRequestContext.self) {
    CORSMiddleware()
    LogRequestsMiddleware(.trace, includeHeaders: .all())
    ErrorMiddleware(templates: templates)

    Route(.get, "", handler: loginPage)
    Route(.get, "index.html", handler: loginPage)

    Route(.post, "login") { request, context in
        struct LoginRequest: Decodable {
            let username: String
            let password: String
        }
        let requestBody = try await request.decode(as: LoginRequest.self, context: context)
        context.logger.debug("TODO requestBody \(requestBody)")
        if case let .some(user) = try await User.validateCredentials(
            on: fluent.db(), username: requestBody.username, password: requestBody.password)
        {
            context.logger.debug("TODO success, user = \(user)")
            return try await loginPage(request: request, context: context)
        } else {
            // TODO show login page again with error message
            throw HTTPError(.unauthorized)
        }
    }

    /*
    TODO real routes
    RouteGroup("user") {
        BasicAuthenticationMiddleware()
        Route(.post, "login") { request, context in
            ...
        }
    }
    */
}

var app = Application(
    router: router,
    configuration: .init(address: .hostname(env.assert("HOST"), port: env.assertInt("PORT"))),
    logger: logger
)

app.beforeServerStarts {
    try await fluent.migrate()

    // TODO testing, remove me
    for user in try await User.query(on: fluent.db()).all() {
        await logger.debug("TODO user: \(user)")
    }
}

do {
    try await app.runService()
} catch {
    logger.critical("server error: \(String(reflecting: error))")
}
