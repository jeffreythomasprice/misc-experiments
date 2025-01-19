import Hummingbird

struct IndexData {
    var content: String
}

func indexView(request: Request, context: any RequestContext, data: () async throws -> IndexData) async throws -> Response {
    let data = try await data()
    return try templates.renderToResponse(data, withTemplate: "index.html")
}
