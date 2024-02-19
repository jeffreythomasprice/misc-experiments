const AUTH_STORAGE_KEY = 'auth'

export function getAuthToken(): string | null {
    return localStorage.getItem(AUTH_STORAGE_KEY)
}

export function setAuthToken(token: string): void {
    localStorage.setItem(AUTH_STORAGE_KEY, token)
}

export function clearAuthToken(): void {
    localStorage.removeItem(AUTH_STORAGE_KEY)
}
