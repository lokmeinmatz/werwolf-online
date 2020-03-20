import {getCurrentTokenString} from './utils'

export function rawApiFetch(url: string, params?: RequestInit): Promise<Response> {
    console.log(`Requesting api from ${url}`)

    const token = getCurrentTokenString()
    if (!token) return Promise.reject("no token stored")

    if (!params) params = {}
    let headers = (!params.headers) ? new Headers() : new Headers(params.headers)
    headers.append("Authorization", `Bearer ${token}`)

    params.headers = headers
    return fetch(`/api/v1${url}`, params)
}

export interface PlayerData {
    name: string,
    role: object | null
}

export async function getPlayerList(sid: string): Promise<PlayerData[]> {
    const url = `/sessions/${sid}/playerlist`

    const res = await rawApiFetch(url)

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()
        return list
    }

    return Promise.reject(`failed to load list from server: ${res.status} (${res.statusText})`)

}

export interface SessionData {
    id: string,
    player_count: number,
    active: boolean,
    created: number
}

exp