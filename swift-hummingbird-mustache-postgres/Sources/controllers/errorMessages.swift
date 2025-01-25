import Elementary

class ErrorMessages: HTML {
    private let messages: [String]?

    init(messages: [String]?) {
        self.messages = messages
    }

    var content: some HTML {
        if let messages = messages {
            for message in messages {
                div(.class("error")) { message }
            }
        }
    }
}
