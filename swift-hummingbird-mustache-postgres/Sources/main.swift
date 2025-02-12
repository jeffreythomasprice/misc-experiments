import Foundation
import Hummingbird
import HummingbirdRouter
import Logging
import PostgresMigrations
import PostgresNIO
import SwiftDotenv
import Synchronization

LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

var logger = Logger(label: "Experiment")
logger.logLevel = .trace

try Dotenv.configure(atPath: "local.env")

let env = Env(logger: logger)

let db = await Database(logger: logger.child(logLevel: .info, label: "DB"), env: env)

let auth: Auth
do {
    auth = try await Auth(logger: logger.child(label: "Auth"))
} catch {
    logger.critical("auth init error: \(error)")
    exit(1)
}

let clicks = ClickActor()

let router = RouterBuilder(context: ExtendedRequestContext.self) {
    CORSMiddleware()
    LogRequestsMiddleware(.trace, includeHeaders: .all())
    ErrorMiddleware()
    FileMiddleware(
        "static", urlBasePath: "/static/", cacheControl: .init([]),
        searchForIndexHtml: false, logger: logger.child(label: "FilesMiddleware"))

    Get { _, _ in
        Response.redirect(to: "/login")
    }
    Get("index.html") { _, _ in
        Response.redirect(to: "/login")
    }

    Get("favicon.ico") { _, _ in Response.redirect(to: "/static/favicon.ico") }

    // TODO real landing page, no click demo
    LoginController(redirectOnSuccessfulLogin: "/auth/click")

    RouteGroup("auth") {
        AuthMiddleware(auth: auth, db: db, redirect: "/login")

        ClickController(clicks: clicks)
        UsersController(db: db)
    }
}

var app = Application(
    router: router,
    configuration: .init(address: .hostname(env.assert("HOST"), port: env.assertInt("PORT"))),
    logger: logger.child(label: "App")
)

app.addServices(await db.client)

app.beforeServerStarts {
    try await db.migrate()
}

do {
    try await app.runService()
} catch {
    logger.critical("server error: \(String(reflecting: error))")
}
