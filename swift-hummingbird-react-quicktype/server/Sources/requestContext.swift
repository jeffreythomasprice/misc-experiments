import Hummingbird
import HummingbirdRouter

final class ExtendedRequestContext: RequestContext, RouterRequestContext, @unchecked Sendable {
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
