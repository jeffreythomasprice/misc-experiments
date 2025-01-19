extension AsyncSequence {
    func toArray() async throws -> [Element] {
        try await reduce(into: [Element]()) { results, elem in
            results.append(elem)
        }
    }
}

struct User {
    var username: String
    var password: String?

    static func listAll(db: Database) async throws -> any AsyncSequence<User, any Error> {
        try await db.client.query("SELECT username FROM users")
            .decode((String).self)
            .map { (username) in
                User(username: username)
            }
    }

    static func validateCredentials(db: Database, username: String, password: String) async throws -> User? {
        try await db.client.query(
            "SELECT username, password FROM users WHERE username = \(username) AND password = \(password)"
        )
        .decode((String, String).self)
        .map { (username, password) in
            User(username: username, password: password)
        }
        .toArray()
        .first
    }

    static func findByUsername(db: Database, username: String) async throws -> User? {
        try await db.client.query(
            "SELECT username FROM users WHERE username = \(username)"
        )
        .decode((String).self)
        .map { (username) in
            User(username: username)
        }
        .toArray()
        .first
    }
}
