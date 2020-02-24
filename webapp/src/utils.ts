interface ClientTokenData {
    type: string
    sub: string,
    session_id: string
}

interface AdminTokenData {
    type: string
}

export function parseJWTokenData<T>(jwt: string | null): T | null {
    if (jwt) {
        const splitted = jwt.split(".")
        
        if (splitted.length != 3) return null

        return JSON.parse(atob(splitted[1]))
    }
    return null
}

export function updateClientToken(token: string) {
    localStorage.setItem("token", token)
    document.cookie = `token=${token}`
}

export function getCurrentClientTokenString() : string | null {
    return localStorage.getItem("token")
}

export function getCurrentClientTokenData() : ClientTokenData | null {
    let jwt = localStorage.getItem("token")
    return parseJWTokenData(jwt)
}

export function updateAdminToken(token: string) {
    localStorage.setItem("admintoken", token)
    document.cookie = `admintoken=${token}`
}

export function getCurrentAdminTokenString() : string | null {
    return localStorage.getItem("admintoken")
}

export function getCurrentAdminTokenData() : AdminTokenData | null {
    let jwt = localStorage.getItem("admintoken")
    return parseJWTokenData(jwt)
}