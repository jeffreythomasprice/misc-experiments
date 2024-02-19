import Vapor

struct LoginRequest: Content {
    var username: String
    var password: String
}

struct LoginResponse: Content {
    var username: String
    var token: String
}

struct ErrorResponse: Content {
    var messages: [String]
}
