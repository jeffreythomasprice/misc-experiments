import Hummingbird

struct IndexData {
    var currentUser: User?
    var content: String
}

func indexView(request: Request, context: ExtendedRequestContext, data: () async throws -> IndexData) async throws -> Response {
    var data = try await data()
    data.currentUser = data.currentUser ?? context.currentUser
    return try templates.renderToResponse(data, withTemplate: "index.html")
}
