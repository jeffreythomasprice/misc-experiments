namespace Shared

module Login =
    type Request = { username: string; password: string }

    type Response = { username: string }
