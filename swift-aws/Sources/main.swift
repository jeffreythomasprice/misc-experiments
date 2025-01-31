import Foundation
import Logging
import SotoECS
import SotoSTS

LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

var logger = Logger(label: "Experiment")
logger.logLevel = .trace

do {
    let config = try await Config(
        contentsOf: URL(fileURLWithPath: FileManager.default.currentDirectoryPath).appending(
            components: "config", "config.yaml"
        ).standardized)

    // TODO pick profile somehow? check exists?
    let profile = config.profiles["main"]!

    let credentialsCache = try FileSystemCache<Credentials>(fileName: "credentials")
    let credentials = try await Credentials(
        logger: logger,
        cache: credentialsCache,
        profile: profile
    )

    // TODO only print export vars if flag says to
    print("export AWS_ACCESS_KEY_ID=\(credentials.accessKeyId)")
    print("export AWS_SECRET_ACCESS_KEY=\(credentials.secretAccessKey)")
    print("export AWS_SESSION_TOKEN=\(credentials.sessionToken)")

    try await doWithClient(client: credentials.createAWSClient()) { client in
        let ecs = ECS(client: client)
        let clusters = try await ecs.listClusters()
        logger.debug("TODO clusters = \(clusters)")
    }
} catch {
    logger.error("fatal: \(error)")
    exit(1)
}
