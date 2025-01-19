import Hummingbird

struct IndexData {
    var currentUser: User?
    var content: String

    init(request: Request, auth: Auth, db: Database, content: String) async {
        // TODO instead of passing request, auth, and db around, can we get all those from request+context?
        currentUser = await auth.getUser(request: request, db: db)
        self.content = content
    }
}

func indexView(request: Request, context: any RequestContext, data: () async throws -> IndexData) async throws -> Response {
    let data = try await data()
    return try templates.renderToResponse(data, withTemplate: "index.html")
}
