import Foundation

struct TimeComponents {
    let days: Int
    let hours: Int
    let minutes: Int
    let seconds: Int
    let subsecond: Double

    init(days: Int, hours: Int, minutes: Int, seconds: Int, subsecond: Double) {
        self.days = days
        self.hours = hours
        self.minutes = minutes
        self.seconds = seconds
        self.subsecond = subsecond
    }

    init(timeInterval: TimeInterval) {
        var remainder = timeInterval
        let days = Int(floor(remainder / TimeInterval.Days))
        remainder -= Double(days) * TimeInterval.Days
        let hours = Int(floor(remainder / TimeInterval.Hours))
        remainder -= Double(hours) * TimeInterval.Hours
        let minutes = Int(floor(remainder / TimeInterval.Minutes))
        remainder -= Double(minutes) * TimeInterval.Minutes
        let seconds = Int(floor(remainder / TimeInterval.Seconds))
        remainder -= Double(seconds) * TimeInterval.Seconds
        let subsecond = remainder
        self.init(days: days, hours: hours, minutes: minutes, seconds: seconds, subsecond: subsecond)
    }
}

extension TimeInterval {
    static var Seconds: Self { 1 }
    static var Minutes: Self { Seconds * 60 }
    static var Hours: Self { Minutes * 60 }
    static var Days: Self { Hours * 24 }

    var components: TimeComponents { .init(timeInterval: self) }

    func humanReadable() -> String {
        let components = self.components
        let days =
            if components.days > 0 {
                "\(components.days) d "
            } else {
                ""
            }
        let microseconds = Int(components.subsecond * 100000).description.leftPad(toLength: 6, withPad: "0")
        return "\(days)\(components.hours):\(components.minutes):\(components.seconds).\(microseconds)"
    }
}

extension String {
    func leftPad<T: StringProtocol>(toLength: Int, withPad padString: T) -> String {
        String(
            String(reversed())
                .padding(toLength: toLength, withPad: padString, startingAt: 0)
                .reversed()
        )
    }
}
