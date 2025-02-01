import ArgumentParser
import Foundation
import Logging
import SotoCloudFormation
import SotoECS
import SotoSTS

@main
struct Main: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "",
        version: "0.0.1",
        subcommands: [
            Env.self,
            ListECSClusters.self,
            DescribeECSCluster.self,
            ListCloudFormationStacks.self,
        ]
    )
}

extension Main {
    struct Options: ParsableArguments {
        @Option(name: .shortAndLong, help: "Which profile in the config file to use")
        var profile: String

        @Option(name: .shortAndLong, help: "Region")
        var region: String

        func getAwsRegion() throws -> Region {
            guard let result = Region.init(awsRegionName: region) else {
                throw ValidationError("Must provide a valid AWS region")
            }
            return result
        }

        func validate() throws {
            _ = try getAwsRegion()
        }
    }

    struct Env: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "Print eval-able env vars")

        @OptionGroup var options: Options

        mutating func run() async throws {
            await Program().env(profile: options.profile, region: try options.getAwsRegion())
        }
    }

    struct ListECSClusters: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "List all ECS clusters")

        @OptionGroup var options: Options

        mutating func run() async throws {
            await Program().listECSClusters(profile: options.profile, region: try options.getAwsRegion())
        }
    }

    struct DescribeECSCluster: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "Describe an ECS cluster")

        @OptionGroup var options: Options

        @Option(name: .customLong("cluster-arn"), help: "Look for cluster by ARN")
        var clusterArn: String?

        @Option(name: .customLong("cluster-name"), help: "Look for cluster by name")
        var clusterName: String?

        @Option(name: .customLong("filter"), help: "Only show service names that contain this string")
        var filter: String?

        func validate() throws {
            if (clusterArn == nil && clusterName == nil) || (clusterArn != nil && clusterName != nil) {
                throw ValidationError("Must provide exactly one of cluster ARN or name")
            }
        }

        mutating func run() async throws {
            if let clusterArn = self.clusterArn {
                await Program().describeECSCluster(
                    profile: options.profile, region: try options.getAwsRegion(), clusterArn: clusterArn, filter: filter)
            } else if let clusterName = self.clusterName {
                await Program().describeECSCluster(
                    profile: options.profile, region: try options.getAwsRegion(), clusterName: clusterName, filter: filter)
            } else {
                throw ValidationError("should be impossible")
            }
        }
    }

    struct ListCloudFormationStacks: AsyncParsableCommand {
        static let configuration = CommandConfiguration(abstract: "List all cloudformation stacks")

        @OptionGroup var options: Options

        @Option(name: .customLong("filter"), help: "Only show stack names that contain this string")
        var filter: String?

        mutating func run() async throws {
            await Program().listCloudFormationStacks(profile: options.profile, region: try options.getAwsRegion(), filter: filter)
        }
    }
}

enum ProfileError: Error {
    case noSuchProfile(String)
}

class Program {
    private let logger: Logger
    private let config: Config
    private let credentialsCache: Cache<Credentials>

    init() async {
        LoggingSystem.bootstrap { name in PrintLogger.init(name: name, destination: SendableTextOutputStream(Stdout())) }

        var logger = Logger(label: "Experiment")
        logger.logLevel = .trace
        self.logger = logger

        do {
            config = try await Config()

            let storage = try FileSystemStorage(fileName: "data")
            credentialsCache = Cache(storage: storage, prefix: "credentials")
        } catch {
            logger.error("init error: \(error)")
            exit(1)
        }
    }

    func env(profile: String, region: Region) async {
        await orExit {
            let credentials = try await getCredentials(profile: profile)
            print("export AWS_ACCESS_KEY_ID=\(credentials.accessKeyId)")
            print("export AWS_SECRET_ACCESS_KEY=\(credentials.secretAccessKey)")
            print("export AWS_SESSION_TOKEN=\(credentials.sessionToken)")
        }
    }

    func listECSClusters(profile: String, region: Region) async {
        await doWithProfile(profile: profile, region: region) { clientFactory in
            for result in try await Paging({ nextToken async throws in
                let results = try await clientFactory.ecs.listClusters(ECS.ListClustersRequest(nextToken: nextToken))
                return (results.nextToken, results.clusterArns)
            }).collect().sorted() {
                print("\(result)")
            }
        }
    }

    func describeECSCluster(profile: String, region: Region, clusterName: String, filter: String?) async {
        await doWithProfile(profile: profile, region: region) { clientFactory in
            let results = try await Paging({ nextToken async throws in
                let results = try await clientFactory.ecs.listClusters(ECS.ListClustersRequest(nextToken: nextToken))
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

            try await describeCluster(ecs: clientFactory.ecs, clusterArn: results[0], filter: filter)
        }
    }

    func describeECSCluster(profile: String, region: Region, clusterArn: String, filter: String?) async {
        await doWithProfile(profile: profile, region: region) { clientFactory in
            try await describeCluster(ecs: clientFactory.ecs, clusterArn: clusterArn, filter: filter)
        }
    }

    func listCloudFormationStacks(profile: String, region: Region, filter: String?) async {
        await doWithProfile(profile: profile, region: region) { clientFactory in
            let stacks = try await Paging({ nextToken async throws in
                let results = try await clientFactory.cloudFormation.describeStacks(
                    CloudFormation.DescribeStacksInput(nextToken: nextToken))
                return (results.nextToken, results.stacks)
            })
            .filter { stack in
                if let filter = filter {
                    stack.stackName?.localizedCaseInsensitiveContains(filter) ?? false
                } else {
                    true
                }
            }
            .collect()
            .sorted { a, b in
                if let a = a.stackName, let b = b.stackName {
                    a.localizedCaseInsensitiveCompare(b) == .orderedAscending
                } else {
                    false
                }
            }
            for stack in stacks {
                let stackName = stack.stackName ?? "<no name>"
                let stackStatus = stack.stackStatus?.description ?? "<no status>"
                let stackStatusReason = stack.stackStatusReason ?? "<no status reason>"
                print("\(stackName), status: \(stackStatus), reason: \(stackStatusReason)")
                // TODO also show cloudformation events, maybe filter to just errors and only for stacks that are rolling back or failed?
            }
        }
    }

    private func describeCluster(ecs: ECS, clusterArn: String, filter: String?) async throws {
        let services = try await Paging({ nextToken async throws in
            let result = try await ecs.listServices(ECS.ListServicesRequest(cluster: clusterArn, nextToken: nextToken))
            return (result.nextToken, result.serviceArns)
        })
        .filter { serviceArn in
            if let filter = filter {
                serviceArn.localizedCaseInsensitiveContains(filter)
            } else {
                true
            }
        }
        .map { serviceArn async throws in
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
        .collect()
        .sorted { (a: ECS.Service, b: ECS.Service) in
            return if let aArn = a.serviceArn, let bArn = b.serviceArn {
                aArn.caseInsensitiveCompare(bArn) == .orderedAscending
            } else {
                true
            }
        }

        let tasks = try await services.concurrentMap { service async throws in
            let taskDefinition: ECS.TaskDefinition? =
                if let taskDefinitionArn = service.taskDefinition {
                    try await ecs.describeTaskDefinition(ECS.DescribeTaskDefinitionRequest(taskDefinition: taskDefinitionArn))
                        .taskDefinition
                } else {
                    nil
                }

            let tasks = try await Paging({ nextToken async throws in
                let result = try await ecs.listTasks(ECS.ListTasksRequest(cluster: clusterArn, serviceName: service.serviceName))
                return (result.nextToken, result.taskArns)
            })
            .map { taskArn async throws in
                let result = try await ecs.describeTasks(ECS.DescribeTasksRequest(cluster: clusterArn, tasks: [taskArn]))
                if let failures = result.failures {
                    if failures.count > 0 {
                        print("failed to find task \(taskArn): \(failures)")
                        exit(1)
                    }
                }
                guard let task = result.tasks?.first else {
                    print("failed to find task \(taskArn)")
                    exit(1)
                }
                return task
            }
            .collect()

            return (service, taskDefinition, tasks)
        }

        let now = Date.now

        for (service, taskDefinition, tasks) in tasks {
            let serviceName = service.serviceName ?? "<failed to get service name>"
            let runningCount = service.runningCount?.description ?? "<failed to get running count>"
            let desiredCount = service.desiredCount?.description ?? "<failed to get desired count>"
            print("service name: \(serviceName), task count: \(runningCount)/\(desiredCount)")

            print("    deployments")
            for deployment in (service.deployments ?? [])
                .sorted(by: { a, b in
                    if let a = a.createdAt, let b = b.createdAt {
                        a.compare(b) == .orderedAscending
                    } else {
                        false
                    }
                })
            {
                let createdAt = deployment.createdAt?.ISO8601Format() ?? "<no created at>"
                let rolloutState = deployment.rolloutState?.description ?? "<no rollout state>"
                let runningCount = deployment.runningCount?.description ?? "<no running count>"
                let desiredCount = deployment.desiredCount?.description ?? "<no desired count>"
                print("        \(createdAt), rollout state: \(rolloutState), tasks: \(runningCount)/\(desiredCount)")
            }

            if let taskDefinition = taskDefinition {
                print("    task definition")
                for containerDefinition in taskDefinition.containerDefinitions ?? [] {
                    print(
                        "        container definition name: \(containerDefinition.name ?? "N/A"), image: \(containerDefinition.image ?? "N/A")"
                    )
                }
            } else {
                print("    <failed to get task definition>")
            }

            print("    tasks")
            for task in tasks {
                let (startedAt, uptime) =
                    if let startedAt = task.startedAt {
                        (startedAt.ISO8601Format(), now.timeIntervalSince(startedAt).humanReadable())
                    } else {
                        ("<no start time>", "<no uptime>")
                    }
                let containers =
                    task.containers?.map { container in
                        "\(container.name ?? "<no container name>")"
                    }
                    .sorted { a, b in
                        a.caseInsensitiveCompare(b) == .orderedAscending
                    }
                    .joined(separator: ", ") ?? "<no containers>"
                print("        started at: \(startedAt), uptime: \(uptime), containers: \(containers)")
            }
        }
    }

    private func doWithProfile(profile: String, region: Region, _ f: (ClientFactory) async throws -> Void) async {
        await orExit {
            let credentials = try await getCredentials(profile: profile)
            try await doWithClient(client: credentials.createAWSClient()) { client in
                try await f(ClientFactory(client: client, region: region))
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

struct ClientFactory {
    private let client: AWSClient
    private let region: Region

    init(client: AWSClient, region: Region) {
        self.client = client
        self.region = region
    }

    var ecs: ECS {
        ECS(client: client, region: region)
    }

    var cloudFormation: CloudFormation {
        CloudFormation(client: client, region: region)
    }
}
