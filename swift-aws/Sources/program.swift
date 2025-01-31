import ArgumentParser
import Foundation
import Logging
import SotoECS
import SotoSTS

@main
struct Main: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "",
        version: "0.0.1",
        subcommands: [
            Env.self,
            ListClusters.self,
        ]
    )
}

extension Main {
    struct Options: ParsableArguments {
        @Option(name: .shortAndLong, help: "Which profile in the config file to use")
        var profile: String
    }

    struct Env: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "Print eval-able env vars")

        @OptionGroup var options: Options

        mutating func run() async {
            await Program().env(profile: options.profile)
        }
    }

    struct ListClusters: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "List all ECS clusters")

        @OptionGroup var options: Options

        mutating func run() async {
            await Program().listClusters(profile: options.profile)
        }
    }
}

enum ProfileError: Error {
    case noSuchProfile(String)
}

class Program {
    private let logger: Logger
    private let config: Config
    private let credentialsCache: any Cache<Credentials>

    init() async {
        LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

        var logger = Logger(label: "Experiment")
        logger.logLevel = .trace
        self.logger = logger

        do {
            config = try await Config(
                contentsOf: URL(fileURLWithPath: FileManager.default.currentDirectoryPath).appending(
                    components: "config", "config.yaml"
                ).standardized)

            credentialsCache = try FileSystemCache<Credentials>(fileName: "credentials")
        } catch {
            logger.error("init error: \(error)")
            exit(1)
        }
    }

    func env(profile: String) async {
        await orExit {
            let credentials = try await getCredentials(profile: profile)
            print("export AWS_ACCESS_KEY_ID=\(credentials.accessKeyId)")
            print("export AWS_SECRET_ACCESS_KEY=\(credentials.secretAccessKey)")
            print("export AWS_SESSION_TOKEN=\(credentials.sessionToken)")
        }
    }

    func listClusters(profile: String) async {
        await orExit {
            let credentials = try await getCredentials(profile: profile)
            try await doWithClient(client: credentials.createAWSClient()) { client in
                let ecs = ECS(client: client)
                let clusters = try await ecs.listClusters()
                // TODO paging, show all
                logger.debug("TODO clusters = \(clusters)")
            }
        }
    }

    private func getCredentials(profile profileName: String) async throws -> Credentials {
        guard let profile = config.profiles[profileName] else {
            throw ProfileError.noSuchProfile(profileName)
        }
        return try await Credentials(
            logger: logger,
            cache: credentialsCache,
            profile: profile
        )
    }

    private func orExit(_ f: () async throws -> Void) async {
        do {
            try await f()
        } catch {
            logger.error("fatal: \(error)")
            exit(1)
        }
    }
}
