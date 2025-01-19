import Logging
import PostgresMigrations
import PostgresNIO

struct V1CreateUserTable: DatabaseMigration {
    func apply(connection: PostgresConnection, logger: Logger) async throws {
        try await connection.query(
            """
            CREATE TABLE users (
                "username" VARCHAR(256) NOT NULL,
                "password" VARCHAR(256) NOT NULL
            )
            """,
            logger: logger
        )
        try await connection.query(
            """
            INSERT INTO users ("username", "password") VALUES (\("admin"), \("admin"))
            """,
            logger: logger
        )
    }

    func revert(connection: PostgresConnection, logger: Logger) async throws {
        try await connection.query("DROP TABLE users", logger: logger)
    }
}
