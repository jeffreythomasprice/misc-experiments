import Foundation

extension TimeInterval {
    static func from(seconds value: Double) -> TimeInterval {
        value
    }

    static func from(minutes value: Double) -> TimeInterval {
        from(seconds: value) * 60
    }

    static func from(hours value: Double) -> TimeInterval {
        from(minutes: value) * 60
    }

    static func from(days value: Double) -> TimeInterval {
        from(hours: value) * 24
    }
}
