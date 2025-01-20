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
                    th { "Is Admin?" }
                    th {}
                    th {}
                }
                for user in users {
                    tr {
                        td { user.username }
                        td { if user.isAdmin { "✓" } else { "X" } }
                        td {
                            if currentUser?.isAdmin == true || user.username == currentUser?.username {
                                a(.href("\(BASE_PATH)/update/\(user.username)")) { "Edit" }
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
            if currentUser?.isAdmin == true {
                a(.href("\(BASE_PATH)/create")) { "New" }
            }
        }
    }
}

private struct CreateRequest: Decodable {
    let username: String
    let password: String
    let isAdmin: String?

    // TODO smarter way to handle checkbox data?
    var isAdminBool: Bool { isAdmin == "on" }
}

private class CreateContent: HTML {
    private let username: String
    private let password: String
    private let isAdmin: Bool
    private let errorMessages: [String]?

    init(username: String = "", password: String = "", isAdmin: Bool = false, errorMessages: [String]? = nil) {
        self.username = username
        self.password = password
        self.isAdmin = isAdmin
        self.errorMessages = errorMessages
    }

    var content: some HTML {
        div {
            div { "Create User" }
            form(.method(.post), .action("\(BASE_PATH)/create")) {
                label(.for("username")) { "Username" }
                input(.name("username"), .type(.text), .placeholder("Username"), .value(username))
                label(.for("password")) { "Password" }
                input(.name("password"), .type(.text), .placeholder("password"), .value(password))
                label(.for("isAdmin")) { "Admin?" }
                input(.name("isAdmin"), .type(.checkbox))
                    .attributes(.checked, when: isAdmin)
                button(.type(.submit)) { "Login" }
                ErrorMessages(messages: errorMessages)
            }
        }
    }
}

private struct UpdateRequest: Decodable {
    let password: String?
    let isAdmin: String?

    // TODO smarter way to handle checkbox data?
    var isAdminBool: Bool { isAdmin == "on" }
}

private class UpdateContent: HTML {
    private let username: String
    private let password: String
    private let isAdmin: Bool
    private let errorMessages: [String]?

    init(username: String, password: String = "", isAdmin: Bool = false, errorMessages: [String]? = nil) {
        self.username = username
        self.password = password
        self.isAdmin = isAdmin
        self.errorMessages = errorMessages
    }

    var content: some HTML {
        div {
            div { "Update User" }
            form(.method(.post), .action("\(BASE_PATH)/update/\(username)")) {
                label(.for("username")) { "Username" }
                div { username }
                label(.for("password")) { "Password" }
                input(.name("password"), .type(.text), .placeholder("password"), .value(password))
                label(.for("isAdmin")) { "Admin?" }
                input(.name("isAdmin"), .type(.checkbox))
                    .attributes(.checked, when: isAdmin)
                button(.type(.submit)) { "Update" }
                ErrorMessages(messages: errorMessages)
            }
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

            Get("create") {
                RequiresAdmin()
                Handle { request, context in
                    return HTMLResponse {
                        IndexPage(context: context, content: CreateContent())
                    }
                }
            }

            Post("create") {
                RequiresAdmin()
                Handle { request, context in
                    let requestBody = try await request.decode(as: CreateRequest.self, context: context)
                    context.logger.debug("create user request username=\(requestBody.username), isAdmin=\(requestBody.isAdminBool)")
                    if try await User.findByUsername(db: db, username: requestBody.username) == nil {
                        try await User(username: requestBody.username, password: requestBody.password, isAdmin: requestBody.isAdminBool)
                            .create(
                                db: db)
                        // TODO when showing next page, show a success message too?
                        return Response.redirect(to: "\(BASE_PATH)")
                    } else {
                        return try HTMLResponse {
                            IndexPage(
                                context: context,
                                content: CreateContent(
                                    username: requestBody.username,
                                    password: requestBody.password,
                                    isAdmin: requestBody.isAdminBool,
                                    errorMessages: [
                                        "User already exists."
                                    ]
                                ))
                        }.response(from: request, context: context)
                    }
                }
            }

            Get("update/:username") { request, context in
                if let username = context.parameters.get("username", as: String.self) {
                    // TODO require admin or username matches current user
                    if let user = try await User.findByUsername(db: db, username: username) {
                        return HTMLResponse {
                            IndexPage(context: context, content: UpdateContent(username: username, isAdmin: user.isAdmin))
                        }
                    } else {
                        throw HTTPError(.notFound)
                    }
                } else {
                    throw HTTPError(.badRequest)
                }
            }

            Post("update/:username") { request, context in
                if let username = context.parameters.get("username", as: String.self) {
                    // TODO require admin or username matches current user
                    let requestBody = try await request.decode(as: UpdateRequest.self, context: context)
                    let password: String? =
                        if let password = requestBody.password {
                            if password.isEmpty {
                                nil
                            } else {
                                password
                            }
                        } else {
                            nil
                        }
                    context.logger.debug(
                        "update user request username=\(username), changing password? \(password != nil), isAdmin=\(requestBody.isAdminBool)"
                    )
                    // TODO if changing admin current user must be admin
                    try await User(username: username, password: password, isAdmin: requestBody.isAdminBool).update(db: db)
                    // TODO when showing next page, show a success message too?
                    return Response.redirect(to: "\(BASE_PATH)")
                } else {
                    throw HTTPError(.badRequest)
                }
            }

            Get("delete/:username") { request, context in
                if let username = context.parameters.get("username", as: String.self) {
                    // TODO require admin or username matches current user
                    try await User.deleteByUsername(db: db, username: username)
                    // TODO when showing next page, show a success message too?
                    return Response.redirect(to: "\(BASE_PATH)")
                } else {
                    throw HTTPError(.badRequest)
                }
            }
        }
    }
}
