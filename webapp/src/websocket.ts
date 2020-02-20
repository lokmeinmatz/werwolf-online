export default class ServerNotifications {
    private ws: WebSocket
    private eventCallbacks: Map<string, () => void>

    constructor() {
        console.log("Connectiong for notifications...")
        let token = localStorage.getItem("token")

        if (token == null) throw "No token - not associated with a session"
        this.ws = new WebSocket(`ws:localhost:3031/${token}`)   

        this.ws.onopen = console.log
        this.ws.onerror = console.error

        this.eventCallbacks = new Map()

        this.ws.onmessage = ev => {
            const cb = this.eventCallbacks.get(ev.data)

            if (cb != undefined) {
                cb()
            }
        }
    }

    // returns false if callback for eventID was allready registered => still overwrite
    registerEvent(eventID: string, callback: () => void): boolean {

        const exists = this.eventCallbacks.has(eventID)

        this.eventCallbacks.set(eventID, callback)

        return !exists
    }
}
