import {getCurrentAdminTokenString, getCurrentClientTokenString} from "./utils"


export enum NotificationType {
    PlayerConnection,
    ControllerConnection
}

function getDisplayName(nt: NotificationType) : string {
    switch (nt) {
        case NotificationType.PlayerConnection:
            return "PlayerConnection"
        case NotificationType.ControllerConnection:
            return "ControllerConnection"
        default:
            break;
    }
}

export class ServerNotifications {
    private ws: WebSocket
    private type: NotificationType
    private eventCallbacks: Map<string, () => void>

    constructor(type: NotificationType) {

        this.type = type

        console.log(`Connectiong for notifications with type ${getDisplayName(type)}...`)
        let token = (type == NotificationType.PlayerConnection) ? getCurrentClientTokenString() : getCurrentAdminTokenString()

        if (token == null) throw "No token - not associated with a session or as admin"
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
