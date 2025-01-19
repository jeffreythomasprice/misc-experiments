import Foundation
import Logging
import SwiftDotenv

struct Env {
    private let logger: Logger

    init(logger: Logger) {
        self.logger = logger
    }

    func assert(_ name: String) -> String {
        if case let .some(result) = Dotenv[name] {
            return result.stringValue
        }
        logger.critical("expected env var \(name)")
        exit(1)
    }

    func assertInt(_ name: String) -> Int {
        if case let .some(result) = Int(assert(name)) {
            return result
        }
        logger.critical("env var isn't an integer \(name)")
        exit(1)
    }
}
