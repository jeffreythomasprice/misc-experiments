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
    let currentUser: User?
    let navBar: NavBar?
    let clicks: Int

    init(context: ExtendedRequestContext, clicks: Int) {
        (self.currentUser, self.navBar) = commonTemplateData(context: context)
        self.clicks = clicks
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
