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

private class ClickData: TemplateData {
    var clicks: Int
    var currentUser: User?

    init(context: ExtendedRequestContext, clicks: Int) {
        self.clicks = clicks
        self.currentUser = context.currentUser
    }
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
    let data = ClickData(
        context: context,
        clicks: clicks
    )
    context.logger.debug("TODO click, current user = \(data.currentUser)")
    return try templates.renderToResponse(data, withTemplate: "clicks.html")
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
