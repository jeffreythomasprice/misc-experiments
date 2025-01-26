import SwiftDotenv

enum EnvError: Error {
	case noSuchEnvironmentVariable(String)
}

func assertEnvVar(_ name: String) throws -> String {
	if let result = Dotenv[name]?.stringValue {
		return result
	} else {
		throw EnvError.noSuchEnvironmentVariable(name)
	}
}
