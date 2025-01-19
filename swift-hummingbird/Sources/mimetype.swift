import Foundation
import Hummingbird
import HummingbirdCore
import HummingbirdRouter

private func parseHeadersIntoMediaTypes(headers: [String], context: any RequestContext) -> [MediaType] {
    return headers.compactMap { header in
        if let result = MediaType(from: header) {
            return result
        } else {
            return nil
        }
    }
}

extension HTTPFields {
    func contentType(context: any RequestContext) throws -> MediaType? {
        let results = parseHeadersIntoMediaTypes(headers: self[values: .contentType], context: context)
        if results.count >= 2 {
            context.logger.warning("multiple content type headers")
            throw HTTPError(.badRequest)
        }
        return results.first
    }

    func accept(context: any RequestContext) -> [MediaType] {
        parseHeadersIntoMediaTypes(headers: self[values: .accept], context: context)
    }
}

struct MIMETypeRequestDecoder: RequestDecoder {
    func mediaType(from request: Request, context: some RequestContext) throws -> MediaType {
        guard let header = request.headers[.contentType] else {
            throw HTTPError(.badRequest)
        }
        guard let mediaType = MediaType(from: header) else {
            throw HTTPError(.badRequest)
        }
        return mediaType
    }

    func decode<T>(_ type: T.Type, from request: Request, context: some RequestContext) async throws -> T
    where T: Decodable {
        guard let mediaType = try request.headers.contentType(context: context) else {
            throw HTTPError(.badRequest)
        }
        switch mediaType {
        case .applicationJson:
            return try await JSONDecoder().decode(type, from: request, context: context)
        case .applicationUrlEncoded:
            return try await URLEncodedFormDecoder().decode(type, from: request, context: context)
        default:
            throw HTTPError(.badRequest)
        }
    }
}

struct MIMETypeResponseEncoder: ResponseEncoder {
    func encode(_ value: some Encodable, from request: Request, context: some RequestContext) throws
        -> HummingbirdCore.Response
    {
        for mediaType in request.headers.accept(context: context) {
            switch mediaType {
            case .applicationJson:
                return try JSONEncoder().encode(value, from: request, context: context)
            case .applicationUrlEncoded:
                return try URLEncodedFormEncoder().encode(value, from: request, context: context)
            default:
                continue
            }
        }
        throw HTTPError(.badRequest)
    }
}

struct MIMETypeAwareRequestContext: RequestContext, RouterRequestContext {
    var coreContext: CoreRequestContextStorage

    init(source: Source) {
        self.coreContext = .init(source: source)
    }

    var requestDecoder: MIMETypeRequestDecoder {
        return MIMETypeRequestDecoder()
    }

    var responseEncoder: MIMETypeResponseEncoder {
        return MIMETypeResponseEncoder()
    }

    var routerContext: RouterBuilderContext = .init()
}
