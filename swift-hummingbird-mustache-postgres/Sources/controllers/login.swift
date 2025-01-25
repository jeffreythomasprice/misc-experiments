import Elementary
import Foundation
import Hummingbird
import HummingbirdElementary
import HummingbirdRouter

private class Content: HTML {
    private let username: String
    private let password: String
    private let errorMessages: [String]?

    init(username: String = "", password: String = "", errorMessages: [String]? = nil) {
        self.username = username
        self.password = password
        self.errorMessages = errorMessages
    }

    var content: some HTML {
        div {
            div { "Login" }
            form(.method(.post), .action("/login")) {
                label(.for("username")) { "Username" }
                input(.name("username"), .type(.text), .placeholder("Username"), .value(username))
                label(.for("password")) { "Password" }
                input(.name("password"), .type(.text), .placeholder("password"), .value(password))
                button(.type(.submit)) { "Login" }
                ErrorMessages(messages: errorMessages)
            }
        }
    }
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
                HTMLResponse {
                    IndexPage(context: context, content: Content())
                }
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
                    return try HTMLResponse {
                        IndexPage(
                            context: context,
                            content: Content(
                                username: requestBody.username,
                                password: requestBody.password,
                                errorMessages: ["Invalid credentials."]
                            )
                        )
                    }.response(from: request, context: context)
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
