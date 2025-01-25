import Logging
import PostgresMigrations
import PostgresNIO
import ServiceLifecycle

actor Database {
    private let logger: Logger
    private let _client: PostgresClient

    init(logger: Logger, env: Env) async {
        self.logger = logger
        _client = PostgresClient(
            configuration: PostgresClient.Configuration(
                host: env.assert("POSTGRES_HOST"),
                port: env.assertInt("POSTGRES_PORT"),
                username: env.assert("POSTGRES_USER"),
                password: env.assert("POSTGRES_PASSWORD"),
                database: env.assert("POSTGRES_DB"),
                tls: .disable
            ))
    }

    var client: PostgresClient { _client }

    func migrate() async throws {
        let migrations = DatabaseMigrations()

        await migrations.add(V1CreateUserTable())

        try await migrations.apply(client: _client, logger: logger, dryRun: false)
    }
}
