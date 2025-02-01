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
            DescribeCluster.self,
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

    struct DescribeCluster: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "Describe an ECS cluster")

        @OptionGroup var options: Options

        @Option(name: .customLong("cluster-arn"), help: "Look for cluster by ARN")
        var clusterArn: String?

        @Option(name: .customLong("cluster-name"), help: "Look for cluster by name")
        var clusterName: String?

        func validate() throws {
            if (clusterArn == nil && clusterName == nil) || (clusterArn != nil && clusterName != nil) {
                throw ValidationError("Must provide exactly one of cluster ARN or name")
            }
        }

        mutating func run() async throws {
            if let clusterArn = self.clusterArn {
                await Program().describeCluster(profile: options.profile, clusterArn: clusterArn)
            } else if let clusterName = self.clusterName {
                await Program().describeCluster(profile: options.profile, clusterName: clusterName)
            } else {
                throw ValidationError("should be impossible")
            }
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
        await doWithProfile(profile: profile) { client in
            let ecs = ECS(client: client)
            for result in try await Paging({ nextToken async throws in
                let results = try await ecs.listClusters(ECS.ListClustersRequest(nextToken: nextToken))
                return (results.nextToken, results.clusterArns)
            }).collect().sorted() {
                print("\(result)")
            }
        }
    }

    func describeCluster(profile: String, clusterName: String) async {
        await doWithProfile(profile: profile) { client in
            let ecs = ECS(client: client)

            let results = try await Paging({ nextToken async throws in
                let results = try await ecs.listClusters(ECS.ListClustersRequest(nextToken: nextToken))
                return (results.nextToken, results.clusterArns)
            }).filter { clusterArn in
                clusterArn.hasSuffix("/\(clusterName)")
            }.collect()

            if results.count > 1 {
                print("multiple clusters found that match name")
                for r in results {
                    print("\(r)")
                }
                exit(1)
            }

            if results.count == 0 {
                print("no clusters found that match name")
                exit(1)
            }

            await describeCluster(profile: profile, clusterArn: results[0])
        }
    }

    func describeCluster(profile: String, clusterArn: String) async {
        await doWithProfile(profile: profile) { client in
            let ecs = ECS(client: client)

            let serviceArns = try await Paging({ nextToken async throws in
                let result = try await ecs.listServices(ECS.ListServicesRequest(cluster: clusterArn, nextToken: nextToken))
                return (result.nextToken, result.serviceArns)
            }).collect().sorted()

            let services = try await serviceArns.concurrentMap { serviceArn async throws in
                let result = try await ecs.describeServices(ECS.DescribeServicesRequest(cluster: clusterArn, services: [serviceArn]))
                if let failures = result.failures {
                    if failures.count > 0 {
                        print("failed to find service \(serviceArn): \(failures)")
                        exit(1)
                    }
                }
                guard let service = result.services?.first else {
                    print("failed to find service \(serviceArn)")
                    exit(1)
                }
                return service
            }

            let taskDefinitionArns = services.compactMap { service in
                service.taskDefinition
            }.uniqued()

            let taskDefinitions = try await taskDefinitionArns.concurrentMap { taskDefinitionArn async throws in
                try await ecs.describeTaskDefinition(ECS.DescribeTaskDefinitionRequest(taskDefinition: taskDefinitionArn)).taskDefinition
            }.compacted()

            for service in services {
                print(
                    "\(service.serviceName ?? "<N/A>"), task count \(service.runningCount?.description ?? "<N/A>")/\(service.desiredCount?.description ?? "<N/A>"), task definition \(service.taskDefinition ?? "<N/A>")"
                )
                if let taskDefinition = taskDefinitions.first { taskDefinition in
                    taskDefinition.taskDefinitionArn == service.taskDefinition
                } {
                    for containerDefinition in taskDefinition.containerDefinitions ?? [] {
                        print("    \(containerDefinition.name ?? "N/A") \(containerDefinition.image ?? "N/A")")
                    }
                }
            }
            print()
        }
    }

    private func doWithProfile(profile: String, _ f: (AWSClient) async throws -> Void) async {
        await orExit {
            let credentials = try await getCredentials(profile: profile)
            try await doWithClient(client: credentials.createAWSClient()) { client in
                try await f(client)
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
