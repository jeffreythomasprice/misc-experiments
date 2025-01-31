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
                for result in try await paging(
                    request: ECS.ListClustersRequest(),
                    getNextPage: { request in
                        try await ecs.listClusters(request)
                    })
                {
                    logger.debug("TODO cluster = \(result)")
                }
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

protocol WithNextToken {
    func withNextToken(nextToken: String) -> Self
}

protocol HasNextToken {
    var nextToken: String? { get }
}

protocol HasDataPage<T> {
    associatedtype T

    var data: [T]? { get }
}

func paging<DataType, RequestType: WithNextToken, ResponseType: HasNextToken & HasDataPage<DataType>>(
    request: RequestType,
    getNextPage: (RequestType) async throws -> ResponseType
) async throws -> [DataType] {
    var results: [DataType] = []
    var nextToken: String? = nil
    while true {
        let nextRequest =
            if let nextToken = nextToken {
                request.withNextToken(nextToken: nextToken)
            } else {
                request
            }
        let result = try await getNextPage(
            nextRequest)
        if let data = result.data {
            results.append(contentsOf: data)
        }
        nextToken = result.nextToken
        if nextToken == nil {
            break
        }
    }
    return results
}

extension ECS.ListClustersRequest: WithNextToken {
    func withNextToken(nextToken: String) -> SotoECS.ECS.ListClustersRequest {
        ECS.ListClustersRequest(maxResults: self.maxResults, nextToken: nextToken)
    }
}

extension ECS.ListClustersResponse: HasNextToken & HasDataPage {
    typealias T = String

    var data: [String]? {
        self.clusterArns
    }
}
