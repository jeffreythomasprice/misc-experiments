import Foundation
import Logging
import SotoSTS

enum CredentialsError: Error {
    case missingResponseCredentials
}

struct Credentials: Codable {
    let accessKeyId: String
    let secretAccessKey: String
    let expiration: Date
    let sessionToken: String

    init(credentials: Credentials) {
        self.accessKeyId = credentials.accessKeyId
        self.secretAccessKey = credentials.secretAccessKey
        self.expiration = credentials.expiration
        self.sessionToken = credentials.sessionToken
    }

    init(credentials: STS.Credentials) {
        self.accessKeyId = credentials.accessKeyId
        self.secretAccessKey = credentials.secretAccessKey
        self.expiration = credentials.expiration
        self.sessionToken = credentials.sessionToken
    }

    init(
        logger: Logger,
        cache: any Cache<Credentials>,
        profile: Profile
    ) async throws {
        try await self.init(
            logger: logger, cache: cache, accessKeyId: profile.awsAccessKey, secretAccessKey: profile.awsSecretAccessKey,
            serialNumber: profile.serialNumber)
    }

    init(
        logger: Logger,
        cache: any Cache<Credentials>,
        accessKeyId: String,
        secretAccessKey: String,
        serialNumber: String
    ) async throws {
        let cacheKey = "\(accessKeyId):\(serialNumber)"
        if let credentials = try await cache.get(key: cacheKey) {
            if !credentials.isExpired {
                self.init(credentials: credentials)
                return
            }
            logger.debug("existing credentials for \(cacheKey) exist, but are expired")
            try await cache.del(key: cacheKey)
        }

        logger.debug("fetching new credentials for \(cacheKey)")
        self.init(
            credentials: try await doWithClient(
                client: AWSClient(
                    credentialProvider: .static(accessKeyId: accessKeyId, secretAccessKey: secretAccessKey))
            ) { tempClient in
                let sts = STS(
                    client: tempClient
                )
                let response = try await sts.getSessionToken(
                    STS.GetSessionTokenRequest(
                        durationSeconds: nil, serialNumber: serialNumber,
                        tokenCode: promptForMFACode(serialNumber: serialNumber)))
                try? await tempClient.shutdown()

                if let credentials = response.credentials {
                    let result = Credentials(credentials: credentials)
                    logger.debug("updated credentials for \(cacheKey), expiration = \(result.expiration)")
                    try await cache.set(
                        key: cacheKey,
                        value: result,
                        expiresAt: credentials.expiration)
                    return result
                } else {
                    logger.error("response for \(cacheKey) did not include credentials")
                    throw CredentialsError.missingResponseCredentials
                }
            })
    }

    var isExpired: Bool {
        expiration < Date.now
    }

    func createAWSClient() -> AWSClient {
        AWSClient(
            credentialProvider: .static(
                accessKeyId: accessKeyId, secretAccessKey: secretAccessKey,
                sessionToken: sessionToken))
    }
}

func doWithClient<T>(client: AWSClient, _ f: (AWSClient) async throws -> T) async throws -> T {
    do {
        let result = try await f(client)
        try? await client.shutdown()
        return result
    } catch {
        try? await client.shutdown()
        throw error
    }
}

private func promptForMFACode(serialNumber: String) -> String {
    print("Enter an MFA code for \(serialNumber)")
    while true {
        if let result = readLine() {
            if !result.isEmpty {
                return result
            }
        }
    }
}
