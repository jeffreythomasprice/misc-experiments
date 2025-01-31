import Foundation

protocol Cache<T> {
    associatedtype T

    func get(key: String) async throws -> T?
    func set(key: String, value: T, expiresAt: Date) async throws
    func del(key: String) async throws
}

enum CacheError: Error {
    case unknown(message: String)
}

struct ExpiringValue<T>: Codable where T: Codable {
    let value: T
    let expiresAt: Date

    var isExpired: Bool {
        expiresAt < Date.now
    }
}

class FileSystemCache<T>: Cache where T: Codable {
    private let fullPath: URL
    private let parentPath: URL

    private var state: [String: ExpiringValue<T>]

    init(fileName: String) throws {
        self.fullPath =
            URL(fileURLWithPath: FileManager.default.currentDirectoryPath).appending(
                components: ".cache", "swift-aws-experiment", fileName
            ).standardized

        var parentPath = fullPath
        parentPath.append(components: "..")
        parentPath.standardize()
        self.parentPath = parentPath

        self.state = [:]
        try self.loadFromFile()
    }

    func get(key: String) async throws -> T? {
        if let result = self.state[key] {
            if result.isExpired {
                try await self.del(key: key)
                return nil
            } else {
                return result.value
            }
        } else {
            return nil
        }
    }

    func set(key: String, value: T, expiresAt: Date) async throws {
        self.state[key] = .init(value: value, expiresAt: expiresAt)
        try self.saveToFile()
    }

    func del(key: String) async throws {
        self.state.removeValue(forKey: key)
        try self.saveToFile()
    }

    private func loadFromFile() throws {
        try makeDir()
        if let data = try? Data(contentsOf: fullPath) {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .iso8601
            self.state = try decoder.decode([String: ExpiringValue<T>].self, from: data)
        }
    }

    private func saveToFile() throws {
        try makeDir()
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        try encoder.encode(state).write(to: fullPath)
    }

    private func makeDir() throws {
        try FileManager.default.createDirectory(
            at: parentPath, withIntermediateDirectories: true)
    }
}
