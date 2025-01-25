import Logging
import PostgresMigrations
import PostgresNIO

struct V1CreateUserTable: DatabaseMigration {
    func apply(connection: PostgresConnection, logger: Logger) async throws {
        try await connection.query(
            """
            CREATE TABLE "users" (
                "username" VARCHAR(256) NOT NULL UNIQUE,
                "password" VARCHAR(256) NOT NULL,
                "isAdmin" BOOLEAN NOT NULL
            )
            """,
            logger: logger
        )
        try await connection.query(
            """
            INSERT INTO "users" ("username", "password", "isAdmin") VALUES (\("admin"), \("admin"), TRUE)
            """,
            logger: logger
        )
    }

    func revert(connection: PostgresConnection, logger: Logger) async throws {
        try await connection.query("DROP TABLE users", logger: logger)
    }
}
