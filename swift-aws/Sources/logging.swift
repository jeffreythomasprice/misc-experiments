import Foundation
import Logging
import Synchronization

struct Stdout: TextOutputStream {
    func write(_ string: String) {
        print(string, terminator: "")
    }
}

class SendableTextOutputStream: TextOutputStream, @unchecked Sendable {
    let mutex: Mutex<TextOutputStream>

    init(_ value: sending TextOutputStream) {
        mutex = .init(value)
    }

    func write(_ string: String) {
        mutex.withLock { $0.write(string) }
    }
}

struct PrintLogger: LogHandler {
    var metadata: Logging.Logger.Metadata = .init()
    var logLevel: Logging.Logger.Level = .info

    private let name: String
    private let destination: SendableTextOutputStream
    private let formatter: DateFormatter

    init(name: String, destination: SendableTextOutputStream) {
        self.name = name
        self.destination = destination

        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.sssZZZZZ"
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        self.formatter = formatter
    }

    subscript(metadataKey key: String) -> Logging.Logger.Metadata.Value? {
        get {
            metadata[key]
        }
        set(newValue) {
            metadata[key] = newValue
        }
    }

    func log(
        level: Logger.Level, message: Logger.Message, metadata: Logger.Metadata?, source: String,
        file: String, function: String, line: UInt
    ) {
        let date = formatter.string(from: Date())
        let metadata =
            self.metadata.map { key, value in "\(key)=\(value)" }
            + (metadata?.map { key, value in "\(key)=\(value)" } ?? [])
        var result = "\(date) \(level) \(name): "
        if !metadata.isEmpty {
            result += metadata.joined(separator: " ")
            result += " "
        }
        result += message.description
        var destination = destination
        print(result, to: &destination)
    }
}

extension Logger {
    func child(logLevel: Level? = nil, label: String? = nil) -> Logger {
        var result =
            if let label = label {
                Logger(label: "\(self.label).\(label)")
            } else {
                self
            }
        if let logLevel = logLevel {
            result.logLevel = logLevel
        } else {
            result.logLevel = self.logLevel
        }
        return result
    }
}
