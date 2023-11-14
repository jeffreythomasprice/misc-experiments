namespace Shared

type GenericFailureResponse = { message: string }

module Login =
    type Request = { username: string; password: string }

    type Response = { username: string; token: string }
