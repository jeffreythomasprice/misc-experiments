import Hummingbird
import HummingbirdRouter

let ROUTE_GROUP_PATH = "users"
let BASE_PATH = "/auth/\(ROUTE_GROUP_PATH)"

private struct UsersTableRow {
    let username: String
    let editUrl: String?
    let deleteUrl: String?
}

private struct UsersTableData: TemplateData {
    let currentUser: User?
    let navBar: NavBar?
    let users: [UsersTableRow]
    let createUrl: String

    init(context: ExtendedRequestContext, users: [User]) {
        (self.currentUser, self.navBar) = commonTemplateData(context: context)
        let currentUser = self.currentUser
        self.users = users.map { user in
            let isCurrentUser = user.username == currentUser?.username
            let isAdmin = currentUser?.isAdmin ?? false
            let editUrl: String? =
                if isAdmin || isCurrentUser {
                    "\(BASE_PATH)/edit/\(user.username)"
                } else {
                    nil
                }
            let deleteUrl: String? =
                if isAdmin && !isCurrentUser {
                    "\(BASE_PATH)/delete/\(user.username)"
                } else {
                    nil
                }
            return UsersTableRow(
                username: user.username,
                editUrl: editUrl,
                deleteUrl: deleteUrl
            )
        }
        self.createUrl = "\(BASE_PATH)/create"
    }
}

private func usersTableView(request: Request, context: ExtendedRequestContext, db: Database) async throws -> Response {
    // TODO paging
    let users = try await User.listAll(db: db).toArray()
    let data = UsersTableData(
        context: context,
        users: users
    )
    return try templates.renderToResponse(data, withTemplate: "users-table-view.html")
}

struct UsersController<Context: ExtendedRequestContext>: RouterController {
    let db: Database

    var body: some RouterMiddleware<ExtendedRequestContext> {
        RouteGroup("\(ROUTE_GROUP_PATH)") {
            Get { request, context in
                try await usersTableView(request: request, context: context, db: db)
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
