import Elementary
import Hummingbird
import HummingbirdElementary
import HummingbirdRouter

private let ROUTE_GROUP_PATH = "users"
private let BASE_PATH = "/auth/\(ROUTE_GROUP_PATH)"

private class TableContent: HTML {
    private let users: [User]
    private let currentUser: User?

    init(context: ExtendedRequestContext, users: [User]) {
        self.users = users
        self.currentUser = context.currentUser
    }

    var content: some HTML {
        div {
            table {
                tr {
                    th { "Username" }
                    th {}
                    th {}
                }
                for user in users {
                    tr {
                        td { user.username }
                        td {
                            if currentUser?.isAdmin == true || user.username == currentUser?.username {
                                a(.href("\(BASE_PATH)/edit/\(user.username)")) { "Edit" }
                            }
                        }
                        td {
                            if currentUser?.isAdmin == true && user.username != currentUser?.username {
                                a(.href("\(BASE_PATH)/delete/\(user.username)")) { "Delete" }
                            }
                        }
                    }
                }
            }
            // TODO paging info
            a(.href("\(BASE_PATH)/create")) { "New" }
        }
    }
}

struct UsersController<Context: ExtendedRequestContext>: RouterController {
    let db: Database

    var body: some RouterMiddleware<ExtendedRequestContext> {
        RouteGroup("\(ROUTE_GROUP_PATH)") {
            Get { request, context in
                let users = try await User.listAll(db: db).toArray()
                return HTMLResponse {
                    IndexPage(context: context, content: TableContent(context: context, users: users))
                }
            }

            // TODO impl
            // Get("create") {
            // }

            // TODO impl
            // Get(":username") {
            // }

            // TODO impl
            // Get("edit/:username") {
            // }

            // TODO impl
            // Post("delete/:username") {
            // }
        }
    }
}
