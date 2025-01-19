import Foundation
import Hummingbird
import HummingbirdRouter

private class LoginData: TemplateData {
    var username: String
    var password: String
    var errorMessages: [String]?

    init(username: String = "", password: String = "", errorMessages: [String]? = nil) {
        self.username = username
        self.password = password
        self.errorMessages = errorMessages
    }

    var currentUser: User? = nil
}

private func loginView(request: Request, context: ExtendedRequestContext, data: LoginData) async throws -> Response {
    try templates.renderToResponse(data, withTemplate: "login.html")
}

private struct LoginRequest: Decodable {
    let username: String
    let password: String
}

struct LoginController<Context: ExtendedRequestContext>: RouterController {
    var redirectOnSuccessfulLogin: String

    var body: some RouterMiddleware<ExtendedRequestContext> {
        RouteGroup("login") {
            Get { request, context in
                try await loginView(
                    request: request, context: context,
                    data: LoginData())
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

        RouteGroup("logout") {
            Get { request, context in
                var response = Response.redirect(to: "login")
                // "delete" cookie by clearing it and setting expiration  to now
                response.setCookie(Cookie(name: "jwt", value: "", expires: Date.now))
                return response
            }
        }
    }
}
