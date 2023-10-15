import SQLite
import Vapor

struct User {
	let username: String
	let password: String
}

enum DBError: Error {
	case tooManyItems
	case unknown(Error)
}

class DBService {
	private let log: Logger?
	private let getUserByName: Statement
	private let getUserByNameAndPassword: Statement
	private let createUser: Statement

	init(log: Logger?, connection: Connection) throws {
		self.log = log

		self.log?.debug("creating table if not exists")
		try connection.execute(
			"""
			create table if not exists users (
				username varchar(256) not null unique,
				password varchar(256) not null
			)
			""")

		self.getUserByName = try connection.prepare(
			"""
			select username, password from users where username = ?
			"""
		)
		// TODO passwords are in plain text
		self.getUserByNameAndPassword = try connection.prepare(
			"""
			select username, password from users where username = ? and password = ?
			"""
		)
		self.createUser = try connection.prepare(
			"""
			insert into users (username, password) values (?, ?)
			"""
		)

		switch self.getUser(username: "admin") {
		case .success(.some(_)):
			self.log?.trace("admin user already exists")
		case .success(.none):
			self.log?.debug("admin user does not exist, creating")
			if case .failure(let e) = self.createUser(User(username: "admin", password: "admin")) {
				throw e
			}
		case .failure(let e):
			throw e
		}
	}

	func getUser(username: String) -> Swift.Result<User?, DBError> {
		do {
			return
				getSingleResult(
					try getUserByName.run(username)
				) { row in
					User(
						username: row[0] as! String,
						password: row[1] as! String
					)
				}
		} catch {
			return .failure(.unknown(error))
		}
	}

	func getUser(username: String, password: String) -> Swift.Result<User?, DBError> {
		do {
			return
				getSingleResult(
					try getUserByNameAndPassword.run(username, password)
				) { row in
					User(
						username: row[0] as! String,
						password: row[1] as! String
					)
				}
		} catch {
			return .failure(.unknown(error))
		}
	}

	func createUser(_ user: User) -> Swift.Result<(), DBError> {
		do {
			try createUser.run(user.username, user.password)
			return .success(())
		} catch {
			return .failure(.unknown(error))
		}
	}

	// TODO delete user
	// TODO update user

	private func getSingleResult(_ s: Statement)
		-> Swift.Result<Statement.Element?, DBError>
	{
		var result: Statement.Element? = .none
		for row in s {
			switch result {
			case .none:
				result = row
			case .some(_):
				return .failure(.tooManyItems)
			}
		}
		return .success(result)
	}

	private func getSingleResult<T>(_ s: Statement, _ f: (Statement.Element) -> T) -> Swift.Result<T?, DBError> {
		return getSingleResult(s).map {
			$0.map { row in
				return f(row)
			}
		}
	}
}
