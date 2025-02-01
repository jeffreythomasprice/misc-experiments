import Foundation
import Yams

struct Profile: Codable {
    let awsAccessKey: String
    let awsSecretAccessKey: String
    let serialNumber: String
}

private enum ConfigError: Error {
    case notFound
}

struct Config: Codable {
    let profiles: [String: Profile]

    init(profiles: [String: Profile]) {
        self.profiles = profiles
    }

    init(contentsOf url: URL) async throws {
        let result: Config = try YAMLDecoder().decode(from: try Data(contentsOf: url))
        self.init(profiles: result.profiles)
    }

    init() async throws {
        if let url = findFile(named: "config.yaml") {
            try await self.init(contentsOf: url)
        } else {
            throw ConfigError.notFound
        }
    }
}
