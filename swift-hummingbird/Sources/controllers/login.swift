import Hummingbird
import HummingbirdRouter

struct LoginData {
    var username: String
    var password: String
    var errorMessages: [String]?
}

func loginView(request: Request, context: any RequestContext, data: LoginData) async throws -> Response {
    try await indexView(request: request, context: context) {
        IndexData(content: try templates.renderToString(data, withTemplate: "login.html"))
    }
}

private struct LoginRequest: Decodable {
    let username: String
    let password: String
}

struct LoginController<Context: RouterRequestContext>: RouterController {
    var redirectOnSuccessfulLogin: String

    var body: some RouterMiddleware<Context> {
        RouteGroup("login") {
            Get { request, context in
                try await loginView(request: request, context: context, data: LoginData(username: "", password: ""))
            }
            Post { request, context in
                let requestBody = try await request.decode(as: LoginRequest.self, context: context)
                context.logger.debug("login request username=\(requestBody.username)")
                if case let .some(user) = try await User.validateCredentials(
                    db: db, username: requestBody.username, password: requestBody.password)
                {
                    context.logger.debug("login success")
                    var response = Response.redirect(to: redirectOnSuccessfulLogin)
                    let (jwtPayload, jwt) = try await auth.sign(user: user)
                    response.setCookie(
                        Cookie(
                            name: "jwt",
                            value: jwt,
                            expires: jwtPayload.exp.value
                        )
                    )
                    return response
                } else {
                    context.logger.debug("login failure")
                    return try await loginView(
                        request: request,
                        context: context,
                        data: LoginData(
                            username: requestBody.username,
                            password: requestBody.password,
                            errorMessages: ["Invalid credentials."]
                        )
                    )
                }
            }
        }
    }
}
