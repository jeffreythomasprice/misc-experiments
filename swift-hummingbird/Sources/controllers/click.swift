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

private func clickView(request: Request, context: ExtendedRequestContext, clicks c: Int? = nil) async throws
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
        IndexData(content: try templates.renderToString(data, withTemplate: "clicks.html"))
    }
}

struct ClickController<Context: ExtendedRequestContext>: RouterController {
    var clicks: ClickActor

    var body: some RouterMiddleware<ExtendedRequestContext> {
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
