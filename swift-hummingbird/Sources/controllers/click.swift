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

func clickView(request: Request, context: any RequestContext, clicks c: Int? = nil) async throws -> Response {
    let c =
        if let c = c {
            c
        } else {
            await clicks.clicks
        }
    return try await indexView(request: request, context: context) {
        IndexData(content: try templates.renderToString(["clicks": c], withTemplate: "clicks.html"))
    }
}

struct ClickController<Context: RouterRequestContext>: RouterController {
    var clicks: ClickActor

    var body: some RouterMiddleware<Context> {
        RouteGroup("click") {
            Get { request, context in
                try await clickView(request: request, context: context)
            }
            Post { request, context in
                return try await clickView(request: request, context: context, clicks: await clicks.increment())
            }
        }
    }
}
