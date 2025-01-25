// TODO remove clicks demo

import Elementary
import Hummingbird
import HummingbirdElementary
import HummingbirdRouter

private class Content: HTML {
    private let clicks: Int

    init(clicks: Int) {
        self.clicks = clicks
    }

    var content: some HTML {
        form(.method(.post), .action("/auth/click")) {
            div { "Clicks: \(clicks)" }
            button(.type(.submit)) { "Click Me" }
        }
    }
}

actor ClickActor {
    var clicks = 0

    func increment() -> Int {
        clicks += 1
        return clicks
    }
}

struct ClickController<Context: ExtendedRequestContext>: RouterController {
    var clicks: ClickActor

    var body: some RouterMiddleware<ExtendedRequestContext> {
        RouteGroup("click") {
            Get { request, context in
                return HTMLResponse {
                    AsyncContent {
                        IndexPage(context: context, content: Content(clicks: await clicks.clicks))
                    }
                }
            }
            Post { request, context in
                return HTMLResponse {
                    AsyncContent {
                        IndexPage(context: context, content: Content(clicks: await clicks.increment()))
                    }
                }
            }
        }
    }
}
