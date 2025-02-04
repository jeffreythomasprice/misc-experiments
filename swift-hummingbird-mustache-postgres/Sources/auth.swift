import FluentKit
import Foundation
import Hummingbird
import JWTKit
import Logging

struct UserJWTPayload: JWTPayload {
    var exp: ExpirationClaim
    var username: String

    init(user: User) {
        exp = .init(value: Date.now.advanced(by: TimeInterval.from(hours: 1)))
        username = user.username
    }

    func verify(using algorithm: some JWTKit.JWTAlgorithm) async throws {
        try self.exp.verifyNotExpired()
    }
}

enum VerifyError: Error {
    case noSuchUser(username: String)
    case missingCookie
}

actor Auth {
    private let logger: Logger
    private let keyCollection: JWTKeyCollection

    init(logger: Logger) async throws {
        self.logger = logger

        let privateKeyStr = try String(contentsOf: URL(fileURLWithPath: "keys/jwt.key"), encoding: .utf8)
        let publicKeyStr = try String(contentsOf: URL(fileURLWithPath: "keys/jwt.key.pub"), encoding: .utf8)
        let privateKey = try Insecure.RSA.PrivateKey(pem: privateKeyStr)
        logger.trace("jwt private key \(privateKey)")
        let publicKey = try Insecure.RSA.PublicKey(pem: publicKeyStr)
        logger.trace("jwt public key \(publicKey)")

        keyCollection = JWTKeyCollection()
        await keyCollection.add(rsa: privateKey, digestAlgorithm: .sha256)
    }

    func sign(user: User) async throws -> (UserJWTPayload, String) {
        let payload = UserJWTPayload(user: user)
        logger.trace("generated payload = \(payload)")
        return (payload, try await keyCollection.sign(payload))
    }

    func verify(jwt: String, db: Database) async throws -> User {
        let payload = try await keyCollection.verify(jwt, as: UserJWTPayload.self, iteratingKeys: false)
        logger.trace("verified payload = \(payload)")
        if let user = try await User.findByUsername(db: db, username: payload.username) {
            return user
        } else {
            throw VerifyError.noSuchUser(username: payload.username)
        }
    }

    func verify(request: Request, db: Database) async throws -> User {
        if let jwtCookie = request.cookies["jwt"] {
            return try await auth.verify(jwt: jwtCookie.value, db: db)
        } else {
            throw VerifyError.missingCookie
        }
    }
}

struct AuthMiddleware<Context: ExtendedRequestContext>: RouterMiddleware {
    private let auth: Auth
    private let db: Database
    private let redirect: String

    init(auth: Auth, db: Database, redirect: String) {
        self.auth = auth
        self.db = db
        self.redirect = redirect
    }

    func handle(
        _ request: Request, context: Context,
        next: (Request, Context) async throws -> Response
    ) async throws -> Response {
        if context.currentUser == nil {
            do {
                let result = try await auth.verify(request: request, db: db)
                context.logger.debug("auth success \(result.username)")
                context.currentUser = result
            } catch {
                context.logger.debug("auth error \(error)")
                return Response.redirect(to: redirect)
            }
        }
        return try await next(request, context)
    }
}

struct RequiresAdmin<Context: ExtendedRequestContext>: RouterMiddleware {
    func handle(_ request: Input, context: Context, next: (Input, Context) async throws -> Output) async throws -> Output {
        if context.currentUser?.isAdmin != true {
            throw HTTPError(.forbidden)
        }
        return try await next(request, context)
    }
}
