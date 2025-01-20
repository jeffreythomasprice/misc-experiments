extension AsyncSequence {
    func toArray() async throws -> [Element] {
        try await reduce(into: [Element]()) { results, elem in
            results.append(elem)
        }
    }
}
