import Foundation

class Cache<T> where T: Codable {
    private let storage: any Storage
    private let prefix: String

    init(storage: any Storage, prefix: String) {
        self.storage = storage
        self.prefix = prefix
    }

    func get(key: String) async throws -> T? {
        try await storage.get(key: getKey(key: key))
    }

    func set(key: String, value: T, expiresAt: Date) async throws {
        try await storage.set(key: getKey(key: key), value: value, expiresAt: expiresAt)
    }

    func del(key: String) async throws {
        try await storage.del(key: getKey(key: key))
    }

    private func getKey(key: String) -> String {
        return "\(prefix)\(key)"
    }
}

protocol Storage {
    func get<T>(key: String) async throws -> T? where T: Decodable
    func set<T>(key: String, value: T, expiresAt: Date) async throws where T: Codable
    func del(key: String) async throws
}

private struct ExpiringValue: Codable {
    enum Error: Swift.Error {
        case encodingError
        case decodingError
    }

    let json: String
    let expiresAt: Date

    init<T>(value: T, expiresAt: Date) throws where T: Encodable {
        guard let json = String(data: try jsonEncoder.encode(value), encoding: .utf8) else {
            throw Error.encodingError
        }
        self.json = json
        self.expiresAt = expiresAt
    }

    func getValue<T>() throws -> T where T: Decodable {
        guard let data = json.data(using: .utf8) else {
            throw Error.encodingError
        }
        return try jsonDecoder.decode(T.self, from: data)
    }

    var isExpired: Bool {
        expiresAt < Date.now
    }

    enum CodingKeys: String, CodingKey {
        case json = "value"
        case expiresAt
    }
}

class FileSystemStorage: Storage {
    private let fullPath: URL
    private let parentPath: URL

    private var state: [String: ExpiringValue]

    init(fileName: String) throws {
        self.fullPath = dataFilePath(named: fileName)

        var parentPath = fullPath
        parentPath.append(components: "..")
        parentPath.standardize()
        self.parentPath = parentPath

        self.state = [:]
        try self.loadFromFile()
    }

    func get<T>(key: String) async throws -> T? where T: Decodable {
        if let result = self.state[key] {
            if result.isExpired {
                try await self.del(key: key)
                return nil
            } else {
                return try result.getValue()
            }
        } else {
            return nil
        }
    }

    func set<T>(key: String, value: T, expiresAt: Date) async throws where T: Encodable {
        self.state[key] = try .init(value: value, expiresAt: expiresAt)
        try self.saveToFile()
    }

    func del(key: String) async throws {
        self.state.removeValue(forKey: key)
        try self.saveToFile()
    }

    private func loadFromFile() throws {
        try makeDir()
        if let data = try? Data(contentsOf: fullPath) {
            self.state = try jsonDecoder.decode([String: ExpiringValue].self, from: data)
            removeAllExpiredKeys()
        }
    }

    private func saveToFile() throws {
        try makeDir()
        removeAllExpiredKeys()
        try jsonEncoder.encode(state).write(to: fullPath)
    }

    private func makeDir() throws {
        try FileManager.default.createDirectory(
            at: parentPath, withIntermediateDirectories: true)
    }

    private func removeAllExpiredKeys() {
        self.state.filter { (_, value) in
            value.isExpired
        }.forEach { (key, _) in
            self.state.removeValue(forKey: key)
        }
    }
}

private var jsonEncoder: JSONEncoder {
    let result = JSONEncoder()
    result.dateEncodingStrategy = .iso8601
    return result
}

private var jsonDecoder: JSONDecoder {
    let result = JSONDecoder()
    result.dateDecodingStrategy = .iso8601
    return result
}
