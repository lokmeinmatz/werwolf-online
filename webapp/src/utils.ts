interface TokenData {
    username: string,
    session_id: string
}

export function parseJWTokenData(jwt: string | null): TokenData | null {
    if (jwt) {
        const splitted = jwt.split(".")
        
        if (splitted.length != 3) return null

        let payload: any = JSON.parse(atob(splitted[1]))
        if (payload && payload.sub && payload.session_id) {
            return {
                username: payload.sub,
                session_id: payload.session_id
            }
        }
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

export function getCurrentClientTokenData() : TokenData | null {
    let jwt = localStorage.getItem("token")
    return parseJWTokenData(jwt)
}

export function updateAdminToken(token: string) {
    localStorage.setItem("token", token)
    document.cookie = `token=${token}`
}

export function getCurrentAdminTokenString() : string | null {
    return localStorage.getItem("token")
}

export function getCurrentAdminTokenData() : TokenData | null {
    let jwt = localStorage.getItem("token")
    return parseJWTokenData(jwt)
}