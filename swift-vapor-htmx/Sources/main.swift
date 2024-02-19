import JWT
import Leaf
import Vapor

struct LoginRequest: Content {
    var username: String
    var password: String
}

struct LoginResponse: Content {
    var username: String
    var password: String
    var errorMessages: [String]
}

struct TokenClaims: JWTPayload {
    enum CodingKeys: String, CodingKey {
        case subject = "sub"
        case expiration = "exp"
        case username = "username"
    }

    var subject: SubjectClaim
    var expiration: ExpirationClaim
    var username: String

    func verify(using signer: JWTKit.JWTSigner) throws {
        try expiration.verifyNotExpired()
    }
}

LoggingSystem.bootstrap { label in
    var result = StreamLogHandler.standardOutput(label: label)
    result.logLevel = .trace
    return result
}

let app = Application()
defer { app.shutdown() }

let db = try DbService()

app.http.server.configuration.port = 8000

app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

app.views.use(.leaf)

app.jwt.signers.use(.hs256(key: "insert secret here"))

app.get { req in
    if let token = req.cookies["auth-token"] {
        if let claims = try? req.jwt.verify(token.string, as: TokenClaims.self) {
            req.logger.info("user token is valid: \(claims)")
            return req.view.render("logged-in")
        }
        req.logger.info("user token provided, but invalid: \(token)")
    } else {
        req.logger.info("user token not provided")
    }
    return req.view.render("login")
}

app.post("login") { req async throws -> Response in
    let request = try req.content.decode(LoginRequest.self)
    req.logger.trace("login request, username = \(request.username)")
    if try db.checkUsernameAndPassword(username: request.username, password: request.password) {
        req.logger.debug("login success, username = \(request.username)")

        let exp = Date.now.addingTimeInterval(60 * 30)
        let token = try req.jwt.sign(
            TokenClaims(
                subject: "experiment",
                expiration: .init(value: exp),
                username: request.username
            ))
        req.logger.trace("token = \(token)")

        let responseBody = try await req.view.render("logged-in")
        let response = try await responseBody.encodeResponse(for: req)
        response.cookies["auth-token"] = HTTPCookies.Value(string: token, expires: exp)
        return response
    } else {
        req.logger.debug("login failure, username = \(request.username)")
        let responseBody = try await req.view.render(
            "login-form",
            LoginResponse(
                username: request.username,
                password: request.password,
                errorMessages: ["Invalid username or password."]
            )
        ).get()
        return try await responseBody.encodeResponse(for: req)
    }
}

app.post("logout") { req async throws -> Response in
    let responseBody = try await req.view.render("login")
    let response = try await responseBody.encodeResponse(for: req)
    response.cookies["auth-token"] = HTTPCookies.Value(string: "", expires: Date.now)
    return response
}

try app.run()
