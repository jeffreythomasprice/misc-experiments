import FluentKit
import SQLKit

struct V1CreateUsersTable: AsyncMigration {
    var name: String = "V1_createUsersTable"

    func prepare(on database: any FluentKit.Database) async throws {
        try await database.schema("users")
            .id()
            .field("username", .string, .required)
            .unique(on: .init(stringLiteral: "username"))
            .field("password", .string, .required)
            .create().get()
        if let sql = database as? SQLDatabase {
            try await sql.insert(into: "users").columns(["id", "username", "password"]).values([
                SQLFunction("gen_random_uuid"), SQLBind("admin"), SQLBind("admin"),
            ])
            .run()
        }
    }

    func revert(on database: any FluentKit.Database) async throws {
        try await database.schema("users").delete().get()
    }
}
