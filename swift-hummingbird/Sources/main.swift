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

let assertEnvVar = { (_ name: String) -> String in
    if case let .some(result) = Dotenv[name] {
        return result.stringValue
    }
    logger.critical("expected env var \(name)")
    exit(1)
}

let assertEnvVarInt = { (_ name: String) -> Int in
    if case let .some(result) = Int(assertEnvVar(name)) {
        return result
    }
    logger.critical("env var isn't an integer \(name)")
    exit(1)
}

let fluent = Fluent(logger: logger)
fluent.databases.use(
    .postgres(
        configuration: .init(
            hostname: assertEnvVar("POSTGRES_HOST"), port: assertEnvVarInt("POSTGRES_PORT"), username: assertEnvVar("POSTGRES_USER"),
            password: assertEnvVar("POSTGRES_PASSWORD"), database: assertEnvVar("POSTGRES_DB"),
            tls: .disable),
        maxConnectionsPerEventLoop: 1, connectionPoolTimeout: .seconds(10), encodingContext: .default,
        decodingContext: .default,
        sqlLogLevel: .debug),
    as: .psql)

struct TestMigration: Migration {
    func prepare(on database: any FluentKit.Database) -> NIOCore.EventLoopFuture<Void> {
        database.schema("users")
            .id()
            .field("email", .string, .required, .sql(unsafeRaw: "unique"))
            .field("password", .string, .required)
            .create()
    }

    func revert(on database: any FluentKit.Database) -> NIOCore.EventLoopFuture<Void> {
        database.schema("users").delete()
    }
}
await fluent.migrations.add(TestMigration())

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

actor ClicksActor {
    var clicks = 0

    func inccrement() {
        clicks += 1
    }
}
let clicks = ClicksActor()

func clicksHandler(request: Request, context: any RequestContext) async throws -> Response {
    return try await index(request: request, context: context) {
        return try templates.renderToString(["clicks": await clicks.clicks], withTemplate: "clicks.html")
    }
}

let router = RouterBuilder(context: BasicRouterRequestContext.self) {
    CORSMiddleware()
    LogRequestsMiddleware(.trace, includeHeaders: .all())
    ErrorMiddleware(templates: templates)

    Route(.get, "", handler: clicksHandler)
    Route(.get, "index.html", handler: clicksHandler)

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
    configuration: .init(address: .hostname(assertEnvVar("HOST"), port: assertEnvVarInt("PORT"))),
    logger: logger
)

app.beforeServerStarts {
    try await fluent.migrate()
}

do {
    try await app.runService()
} catch {
    logger.critical("server error: \(error)")
}
