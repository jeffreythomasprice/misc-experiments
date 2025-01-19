import FluentKit

final class User: Model, @unchecked Sendable {
    static let schema: String = "users"

    @ID(key: .id)
    var id: UUID?

    @Field(key: "username")
    var username: String

    @Field(key: "password")
    var password: String

    init() {}

    init(id: UUID? = nil, username: String, password: String) {
        self.id = id
        self.username = username
        self.password = password
    }

    static func validateCredentials(on: any Database, username: String, password: String) async throws -> User? {
        try await User.query(on: on)
            .filter(\.$username == username)
            .filter(\.$password == password)
            .first()
            .get()
    }
}
