class DirtyRegion {
    struct Range {
        let index: Int
        let count: Int

        init?(index: Int, count: Int) {
            if count < 1 {
                return nil
            }
            self.index = index
            self.count = count
        }

        var min: Int { index }
        var max: Int { index + count - 1 }
    }

    // TODO should be using an actual sparse set or list or something
    private var range: Range? = .none

    var isDirty: Bool { range != nil }

    func append(index: Int) {
        append(range: Range(index: index, count: 1)!)
    }

    func append(range: Range) {
        if let existing = self.range {
            let min = min(existing.min, range.min)
            let max = max(existing.max, range.max)
            self.range = .some(Range(index: min, count: max - min + 1)!)
        } else {
            self.range = .some(range)
        }
    }

    func clear() -> [Range] {
        if let existing = range {
            let result = [existing]
            range = .none
            return result
        } else {
            return []
        }
    }
}

// TODO tests
