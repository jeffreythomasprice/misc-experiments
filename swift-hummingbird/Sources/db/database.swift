import FluentKit
import HummingbirdFluent
import Logging

func initDb(logger: Logger, env: Env) async -> Fluent {
    let fluent = Fluent(logger: logger)
    fluent.databases.use(
        .postgres(
            configuration: .init(
                hostname: env.assert("POSTGRES_HOST"), port: env.assertInt("POSTGRES_PORT"), username: env.assert("POSTGRES_USER"),
                password: env.assert("POSTGRES_PASSWORD"), database: env.assert("POSTGRES_DB"),
                tls: .disable),
            maxConnectionsPerEventLoop: 1, connectionPoolTimeout: .seconds(10), encodingContext: .default,
            decodingContext: .default,
            sqlLogLevel: .debug),
        as: .psql)

    await fluent.migrations.add(V1CreateUsersTable())

    return fluent
}
