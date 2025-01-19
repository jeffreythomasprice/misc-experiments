// TODO remove clicks demo

import Hummingbird
import HummingbirdRouter

actor ClickActor {
    var clicks = 0

    func increment() -> Int {
        clicks += 1
        return clicks
    }
}

private struct ClickData {
    var clicks: Int
}

private func clickView(request: Request, context: any RequestContext, auth: Auth, db: Database, clicks c: Int? = nil) async throws
    -> Response
{
    let clicks =
        if let c = c {
            c
        } else {
            await clicks.clicks
        }
    let data = ClickData(clicks: clicks)
    return try await indexView(request: request, context: context) {
        await IndexData(request: request, auth: auth, db: db, content: try templates.renderToString(data, withTemplate: "clicks.html"))
    }
}

struct ClickController<Context: RouterRequestContext>: RouterController {
    var auth: Auth
    var db: Database
    var clicks: ClickActor

    var body: some RouterMiddleware<Context> {
        RouteGroup("click") {
            Get { request, context in
                try await clickView(request: request, context: context, auth: auth, db: db)
            }
            Post { request, context in
                return try await clickView(request: request, context: context, auth: auth, db: db, clicks: await clicks.increment())
            }
        }
    }
}
