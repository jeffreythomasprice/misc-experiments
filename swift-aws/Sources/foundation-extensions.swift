import Foundation

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
