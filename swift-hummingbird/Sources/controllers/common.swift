import Elementary

struct NavBarItem {
    let text: String
    let url: String
}

class IndexPage<Content: HTML>: HTMLDocument {
    private let currentUser: User?
    private let navBar: [NavBarItem]?
    private let _body: Content

    init(context: ExtendedRequestContext, content: Content) {
        currentUser = context.currentUser
        if currentUser != nil {
            navBar = [
                NavBarItem(text: "Clicks", url: "/auth/click"),
                NavBarItem(text: "Users", url: "/auth/users"),
            ]
        } else {
            navBar = nil
        }
        _body = content
    }

    var title: String = "Experiment"

    var head: some HTML {
        meta(.charset("utf-8"))
        link(.rel("stylesheet"), .href("/static/index.css"))
    }

    var body: some HTML {
        if let currentUser = currentUser {
            div {
                "Logged in as: \(currentUser.username)"
                a(.href("/logout")) { "Log Out" }
            }
        }

        if let navBar = navBar {
            div {
                for item in navBar {
                    a(.href(item.url)) { item.text }
                }
            }
        }

        _body
    }
}
