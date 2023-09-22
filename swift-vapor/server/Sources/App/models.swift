import Vapor

struct LoginRequest: Content {
    var name: String
}

struct LoginResponse: Content {
    var id: String
}

enum CustomJsonError: Error {
    case invalidKey(String)
}

enum ClientToServerMessage: Codable {
    struct Send: Codable {
        let message: String
    }

    struct Login: Codable {
        let id: String
    }

    case send(Send)
    case login(Login)

    enum CodingKeys: String, CodingKey {
        case type
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "send":
            self = .send(try Send(from: decoder))
        case "login":
            self = .login(try Login(from: decoder))
        default:
            throw CustomJsonError.invalidKey(type)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .send(let value):
            try container.encode("send", forKey: .type)
            try value.encode(to: encoder)
        case .login(let value):
            try container.encode("login", forKey: .type)
            try value.encode(to: encoder)
        }
    }
}

enum ServerToClientMessage: Codable {
    struct Send: Codable {
        let senderId: String
        let message: String
    }

    case send(Send)

    enum CodingKeys: String, CodingKey {
        case type
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "send":
            self = .send(try Send(from: decoder))
        default:
            throw CustomJsonError.invalidKey(type)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .send(let value):
            try container.encode("send", forKey: .type)
            try value.encode(to: encoder)
        }
    }
}
