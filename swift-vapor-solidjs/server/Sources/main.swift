import JWT
import Vapor

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

app.http.server.configuration.port = 8001

app.middleware.use(
    CORSMiddleware(
        configuration: CORSMiddleware.Configuration(
            allowedOrigin: .all,
            allowedMethods: [.GET, .POST, .PUT, .DELETE, .PATCH, .OPTIONS],
            allowedHeaders: ["*"]
        )),
    at: .beginning
)

app.jwt.signers.use(.hs256(key: "insert secret here"))

app.get("userInfo") { req async throws in
    if let token = req.headers.bearerAuthorization?.token {
        req.logger.info("user token provided: \(token)")
        if let claims = try? req.jwt.verify(token, as: TokenClaims.self) {
            req.logger.info("user token provided, and is valid: \(token), claims: \(claims)")
            let responseBody = LoginResponse(username: claims.username, token: token)
            return try await responseBody.encodeResponse(for: req)
        } else {
            req.logger.info("user token provided, but is not valid: \(token)")
        }
    } else {
        req.logger.info("user token not provided")
    }
    let responseBody = ErrorResponse(messages: ["Provided token is invalid."])
    let response = try await responseBody.encodeResponse(for: req)
    response.status = .unauthorized
    return response
}

app.post("login") { req async throws in
    let request = try req.content.decode(LoginRequest.self)
    req.logger.trace("login request, username = \(request.username)")
    if try db.checkUsernameAndPassword(username: request.username, password: request.password) {
        let exp = Date.now.addingTimeInterval(60 * 30)
        let token = try req.jwt.sign(
            TokenClaims(
                subject: "experiment",
                expiration: .init(value: exp),
                username: request.username
            ))
        req.logger.trace("token = \(token)")

        let responseBody = LoginResponse(username: request.username, token: token)
        return try await responseBody.encodeResponse(for: req)
    } else {
        let responseBody = ErrorResponse(messages: ["Invalid username or password."])
        let response = try await responseBody.encodeResponse(for: req)
        response.status = .unauthorized
        return response
    }
}

try app.run()
