import Foundation
import Logging
import SotoECS
import SotoSTS
import SwiftDotenv

LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

var logger = Logger(label: "Experiment")
logger.logLevel = .trace

do {
    // TODO make environment configurable? support both at once?
    try Dotenv.configure(atPath: "env/servicepower.env")
    // try Dotenv.configure(atPath: "env/servicepower-sdlc.env")

    let AWS_ACCESS_KEY_ID = try assertEnvVar("AWS_ACCESS_KEY_ID")
    let AWS_SECRET_ACCESS_KEY = try assertEnvVar("AWS_SECRET_ACCESS_KEY")
    let SERIAL_NUMBER = try assertEnvVar("SERIAL_NUMBER")

    let credentialsCache = try FileSystemCache<Credentials>(fileName: "credentials")
    let credentials = try await Credentials(
        logger: logger,
        cache: credentialsCache,
        accessKeyId: AWS_ACCESS_KEY_ID,
        secretAccessKey: AWS_SECRET_ACCESS_KEY,
        serialNumber: SERIAL_NUMBER
    )

    try await doWithClient(client: credentials.createAWSClient()) { client in
        let ecs = ECS(client: client)
        let clusters = try await ecs.listClusters()
        logger.debug("TODO clusters = \(clusters)")
    }
} catch {
    logger.error("fatal: \(error)")
    exit(1)
}
