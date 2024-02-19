import { clearAuthToken, getAuthToken, setAuthToken } from './auth'
import { type ErrorResponse, type LoginRequest, type LoginResponse } from './models'

export class HttpError extends Error {
    constructor(
        readonly statusCode: number,
        readonly messages: string[],
    ) {
        super(`Error(statusCode=${statusCode}, messages=[${messages.join(', ')}]`)
    }

    static unwrap(e: unknown): HttpError {
        if (e instanceof HttpError) {
            return e
        }
        if (e instanceof Error) {
            return new HttpError(0, [e.message])
        }
        if (Array.isArray(e)) {
            return new HttpError(0, e.map(x => HttpError.unwrap(x)).flatMap(e => e.messages))
        }
        if (typeof e === 'string' || typeof e === 'number' || e === undefined || e === null) {
            return new HttpError(0, [`${e}`])
        }
        return new HttpError(0, [JSON.stringify(e)])
    }
}

export async function userInfo(): Promise<LoginResponse | null> {
    try {
        if (!getAuthToken()) {
            return null
        }
        return await jsonRequest<unknown, LoginResponse>('/userInfo', 'GET')
    } catch {
        clearAuthToken()
        return null
    }
}

export async function login(request: LoginRequest): Promise<LoginResponse> {
    clearAuthToken()
    const response = await jsonRequest<LoginRequest, LoginResponse>('/login', 'POST', request)
    setAuthToken(response.token)
    return response
}

async function jsonRequest<RequestType, ResponseType>(path: string, method: string, request?: RequestType): Promise<ResponseType> {
    const headers: HeadersInit = {}
    if (getAuthToken()) {
        headers.Authorization = `Bearer ${getAuthToken()}`
    }
    if (request) {
        headers['Content-Type'] = 'application/json'
    }
    const response = await fetch(
        new URL(path, 'http://localhost:8001/'),
        {
            method,
            headers,
            body: JSON.stringify(request),
        },
    )
    if (response.status < 200 || response.status >= 300) {
        const responseBodyStr = await response.text()
        let responseBody: ErrorResponse
        try {
            responseBody = JSON.parse(responseBodyStr)
            if (!Array.isArray(responseBody.messages)) {
                throw new Error()
            }
        } catch {
            responseBody = {
                messages: [
                    responseBodyStr,
                ],
            }
        }
        throw new HttpError(response.status, responseBody.messages)
    }
    return await response.json()
}
