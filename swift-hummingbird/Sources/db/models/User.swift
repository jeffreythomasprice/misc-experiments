// TODO move paging stuff
struct Paging {
    let limit: Int
    let offset: Int

    init(limit: Int, offset: Int) {
        self.limit = limit
        self.offset = offset
    }

    init(limit: Int, page: Int) {
        self.init(limit: limit, offset: limit * (page - 1))
    }

    var pageIndex: Int {
        offset / limit + 1
    }
}

struct PagingResults<T> {
    let paging: Paging
    let totalCount: Int
    let results: [T]

    var pageIndex: Int {
        paging.pageIndex
    }

    var pageCount: Int {
        let pageCount = totalCount / paging.limit
        return if pageCount * paging.limit < totalCount {
            pageCount + 1
        } else {
            pageCount
        }
    }

    var firstPage: Paging? {
        if pageCount > 1 && pageIndex > 1 {
            Paging(limit: paging.limit, page: 1)
        } else {
            nil
        }
    }

    var previousPage: Paging? {
        if pageIndex <= 1 {
            nil
        } else {
            Paging(limit: paging.limit, page: pageIndex - 1)
        }
    }

    var nextPage: Paging? {
        if pageIndex >= pageCount {
            nil
        } else {
            Paging(limit: paging.limit, page: pageIndex + 1)
        }
    }

    var lastPage: Paging? {
        if pageCount >= 1 && pageIndex < pageCount {
            Paging(limit: paging.limit, page: pageCount)
        } else {
            nil
        }
    }
}

struct User {
    var username: String
    var password: String?
    var isAdmin: Bool

    static func listAll(db: Database, paging: Paging) async throws -> PagingResults<User> {
        let count: Int! = try await db.client.query(
            """
            SELECT COUNT(*) FROM "users"
            """
        )
        .decode(Int.self)
        .toArray()
        .first
        let results = try await db.client.query(
            """
            SELECT "username", "isAdmin" FROM "users" ORDER BY "username" LIMIT \(paging.limit) OFFSET \(paging.offset)
            """
        )
        .decode((String, Bool).self)
        .map { (username, isAdmin) in
            User(username: username, isAdmin: isAdmin)
        }
        .toArray()
        return PagingResults(paging: paging, totalCount: count, results: results)
    }

    static func validateCredentials(db: Database, username: String, password: String) async throws -> User? {
        try await db.client.query(
            """
            SELECT "username", "password", "isAdmin" FROM "users" WHERE "username" = \(username) AND "password" = \(password)
            """
        )
        .decode((String, String, Bool).self)
        .map { (username, password, isAdmin) in
            User(username: username, password: password, isAdmin: isAdmin)
        }
        .toArray()
        .first
    }

    static func findByUsername(db: Database, username: String) async throws -> User? {
        try await db.client.query(
            """
            SELECT "username", "isAdmin" FROM "users" WHERE "username" = \(username)
            """
        )
        .decode((String, Bool).self)
        .map { (username, isAdmin) in
            User(username: username, isAdmin: isAdmin)
        }
        .toArray()
        .first
    }

    func create(db: Database) async throws {
        try await db.client.query(
            """
            INSERT INTO "users" ("username", "password", "isAdmin") VALUES (\(username), \(password), \(isAdmin))
            """
        )
    }

    func update(db: Database) async throws {
        if password == nil || password?.isEmpty == true {
            try await db.client.query(
                """
                UPDATE "users" SET "isAdmin" = \(isAdmin) WHERE "username" = \(username)
                """
            )
        } else {
            try await db.client.query(
                """
                UPDATE "users" SET "password" = \(password), "isAdmin" = \(isAdmin) WHERE "username" = \(username)
                """
            )
        }
    }

    static func deleteByUsername(db: Database, username: String) async throws {
        try await db.client.query(
            """
            DELETE FROM "users" WHERE "username" = \(username)
            """
        )
    }
}
