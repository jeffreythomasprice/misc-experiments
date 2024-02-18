import Logging
import SQLite

enum DbErrors: Error {
    case tooManyRows
}

class DbService {
    private var conn: Connection

    private var usersTable: Table
    private var usersUsernameColumn: Expression<String>
    private var usersPasswordColumn: Expression<String>

    init() throws {
        let logger = Logger(label: "db")
        conn = try Connection("local.sqlite")
        conn.trace { msg in
            logger.trace("sql: \(msg)")
        }

        usersTable = Table("users")
        usersUsernameColumn = Expression<String>("username")
        usersPasswordColumn = Expression<String>("password")

        do {
            let result = try conn.run(
                usersTable.create(ifNotExists: true) { t in
                    t.column(usersUsernameColumn, primaryKey: true)
                    t.column(usersPasswordColumn)
                }
            )
            logger.info("create users table: \(result)")
            for row in try conn.prepare("SELECT sql FROM sqlite_schema WHERE name = 'users'") {
                logger.info("\(row)")
            }
        }

        if try conn.scalar(usersTable.count.filter(usersUsernameColumn == "user")) == 0 {
            logger.info("creating default user")
            try conn.run(usersTable.insert(usersUsernameColumn <- "user", usersPasswordColumn <- "password"))
        }
    }

    func checkUsernameAndPassword(username: String, password: String) throws -> Bool {
        let results = Array(
            try conn.prepare(
                usersTable
                    .select(usersUsernameColumn)
                    .filter(usersUsernameColumn == username && usersPasswordColumn == password)
            ))
        return switch results.count {
        case 0: false
        case 1: true
        default: throw DbErrors.tooManyRows
        }
    }
}
