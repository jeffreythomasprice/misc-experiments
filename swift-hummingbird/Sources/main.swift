import Foundation
import Hummingbird
import HummingbirdRouter
import Logging
import Mustache
import Synchronization

LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

var logger = Logger(label: "Experiment")
logger.logLevel = .trace

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
    // TODO not found middleware

    Route(.get, "", handler: clicksHandler)
    Route(.get, "index.html", handler: clicksHandler)

    Route(.post, "click") { request, context in
        await clicks.inccrement()
        return try await clicksHandler(request: request, context: context)
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

let app = Application(
    router: router,
    configuration: .init(address: .hostname("127.0.0.1", port: 8000)),
    logger: logger
)

do {
    try await app.runService()
} catch {
    logger.critical("server error: \(error)")
}
