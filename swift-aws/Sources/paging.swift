extension AsyncSequence {
    func collect() async throws -> [Element] {
        try await reduce(into: [Element]()) { results, next in
            results.append(next)
        }
    }
}

struct Paging<NextTokenType, DataType>: AsyncSequence, AsyncIteratorProtocol {
    typealias Element = DataType

    typealias AsyncIterator = Self

    private let f: (NextTokenType?) async throws -> (NextTokenType?, [DataType]?)

    private var nextToken: NextTokenType? = nil
    private var currentPage: [DataType]? = nil
    private var iterator: (any IteratorProtocol<DataType>)? = nil

    init(_ f: @escaping (NextTokenType?) async throws -> (NextTokenType?, [DataType]?)) async throws {
        self.f = f

        let (firstToken, firstPage) = try await f(nil)
        self.nextToken = firstToken
        self.currentPage = firstPage
        self.iterator = firstPage?.makeIterator()
    }

    func makeAsyncIterator() -> Paging<NextTokenType, DataType> {
        self
    }

    mutating func next() async throws -> DataType? {
        // if we have stuff left on our current page we can return that
        if let result = self.iterator?.next() {
            return result
        }

        // get the next page if there is one
        if let nextToken = self.nextToken {
            let (nextToken, nextPage) = try await f(nextToken)
            self.nextToken = nextToken
            self.currentPage = nextPage
            self.iterator = self.currentPage?.makeIterator()
        }

        // one more attempt to return the next iterator result
        if let result = self.iterator?.next() {
            return result
        }

        // we definitely don't have any results left
        return nil
    }
}
