import Foundation
import Hummingbird
import HummingbirdRouter
import Logging
import Mustache
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

let templates: Templates
do {
    templates = try await Templates.init(logger: logger.child(label: "Templates"), directory: "templates", withExtension: "mustache")
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

    func increment() {
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

struct LoginData {
    var username: String
    var password: String
    var errorMessages: [String]?
}

func loginPage(request: Request, context: any RequestContext, data: LoginData) async throws -> Response {
    try await index(request: request, context: context) {
        try templates.renderToString(data, withTemplate: "login.html")
    }
}

let router = RouterBuilder(context: MIMETypeAwareRequestContext.self) {
    CORSMiddleware()
    LogRequestsMiddleware(.trace, includeHeaders: .all())
    ErrorMiddleware(templates: templates)
    FileMiddleware(
        "static", urlBasePath: "/static/", cacheControl: .init([]),
        searchForIndexHtml: false, logger: logger.child(label: "FilesMiddleware"))

    Route(.get, "") { _, _ in
        Response.redirect(to: "/login")
    }
    Route(.get, "index.html") { _, _ in
        Response.redirect(to: "/login")
    }

    Route(.get, "favicon.ico") { _, _ in Response.redirect(to: "/static/favicon.ico") }

    Route(.get, "login") { request, context in
        try await loginPage(request: request, context: context, data: LoginData(username: "", password: ""))
    }

    Route(.post, "login") { request, context in
        struct LoginRequest: Decodable {
            let username: String
            let password: String
        }
        let requestBody = try await request.decode(as: LoginRequest.self, context: context)
        context.logger.debug("login request username=\(requestBody.username)")
        if case let .some(user) = try await User.validateCredentials(
            db: db, username: requestBody.username, password: requestBody.password)
        {
            context.logger.debug("login success")
            // TODO real landing page, no click demo
            var response = Response.redirect(to: "/auth/click")
            let (jwtPayload, jwt) = try await auth.sign(user: user)
            response.setCookie(
                Cookie(
                    name: "jwt",
                    value: jwt,
                    expires: jwtPayload.exp.value
                )
            )
            return response
        } else {
            context.logger.debug("login failure")
            return try await loginPage(
                request: request,
                context: context,
                data: LoginData(
                    username: requestBody.username,
                    password: requestBody.password,
                    errorMessages: ["Invalid credentials."]
                )
            )
        }
    }

    RouteGroup("auth") {
        AuthMiddleware(auth: auth, db: db, redirect: "/login")

        // TODO no click demo
        Route(.get, "click") { request, context in
            try await clicksHandler(request: request, context: context)
        }
        Route(.post, "click") { request, context in
            await clicks.increment()
            return try await clicksHandler(request: request, context: context)
        }
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
