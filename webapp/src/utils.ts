interface BasicAuthData {
    exp: number,
    auth_level: "player" | "control"
}

interface PlayerAuthData extends BasicAuthData {
    session_id: string,
    user_name: string,
    role: string | null,
    state: string
}

interface MaybePlayerAuthData extends BasicAuthData {
    session_id?: string,
    user_name?: string,
    role?: string,
    state?: string
}

interface AdminAuthData extends BasicAuthData {
}

interface MaybeAdminAuthData extends BasicAuthData {
}

function toPlayerAuthData(basic: MaybePlayerAuthData): PlayerAuthData | null {
    if(basic.auth_level != "player") return null
    if(basic.session_id == undefined || typeof basic.session_id != "string") return null
    if(basic.user_name == undefined || typeof basic.user_name != "string") return null
    //if(basic.role == undefined || typeof basic.role != "string") return null
    if(basic.state == undefined || typeof basic.state != "string") return null

    return {
        session_id: basic.session_id,
        user_name: basic.user_name,
        role: basic.role,
        state: basic.state,
        ...basic
    }
}

function toAdminAuthData(basic: MaybeAdminAuthData): AdminAuthData | null {
    if(basic.auth_level != "control") return null
    
    return {
        ...basic
    }
}

export function parseJWTokenData<T>(jwt: string | null): T | null {
    if (jwt) {
        const splitted = jwt.split(".")
        
        if (splitted.length != 3) return null

        return JSON.parse(atob(splitted[1]))
    }
    return null
}

export function updateToken(token: string) {
    localStorage.setItem("token", token)
    document.cookie = `token=${token}`
}

export function getCurrentTokenString() : string | null {
    return localStorage.getItem("token")
}

export function getCurrentPlayerTokenData() : PlayerAuthData | null {
    const jwt = localStorage.getItem("token")
    return toPlayerAuthData(parseJWTokenData<BasicAuthData>(jwt))
     
}

export function getCurrentAdminTokenData() : AdminAuthData | null {
    let jwt = localStorage.getItem("admintoken")
    return toAdminAuthData(parseJWTokenData<BasicAuthData>(jwt))
}
export function apiFetch(url: string): Promise<Response> {
    console.log(`Requesting api from ${url}`)

    const req_headers = new Headers()
    const token = getCurrentTokenString()
    req_headers.append("Authorization", `Bearer ${token}`)
    return fetch(`/api/v1${url}`, {
        headers: req_headers
    })
}