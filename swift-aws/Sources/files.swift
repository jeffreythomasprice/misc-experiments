import Foundation

private let DATA_DIR = "swift-aws-tool"
private let HIDDEN_DATA_DIR = ".\(DATA_DIR)"

func findFile(named name: String) -> URL? {
    [
        Bundle.main.executableURL?.appending(components: "..", DATA_DIR),
        FileManager.default.homeDirectoryForCurrentUser.appending(components: HIDDEN_DATA_DIR),
        URL(fileURLWithPath: FileManager.default.currentDirectoryPath).appending(components: DATA_DIR),
        tempDir(),
    ]
    .compactMap { path in
        path?.standardized
    }
    .map { path in
        path.appending(component: name)
    }
    .first { path in
        FileManager.default.fileExists(atPath: path.path)
    }
}

func dataFilePath(named name: String) -> URL {
    tempDir().appending(component: name)
}

private func tempDir() -> URL {
    FileManager.default.temporaryDirectory.appending(components: DATA_DIR)
}
