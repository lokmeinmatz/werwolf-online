import {ServerNotifications, NotificationType} from "../websocket"
import {getCurrentAdminTokenString} from "../utils"
import { ExtendableList } from "../ui"

if (getCurrentAdminTokenString() == null) {
    
    window.location.assign(`/ctrl/?error=NoToken`)
}

let sessionListDom: ExtendableList<SessionData>

async function updateSessionList() {

    const token = getCurrentAdminTokenString()

    if(!token) return

    console.log("requesting session list")

    let req_headers = new Headers()
    req_headers.append("Authorization", `Bearer ${token}`)
    const res = await fetch("/api/v1/session/playerlist", {
        headers: req_headers
    })

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()

        sessionListDom.setData(list)
    }
}

interface SessionData {
    id: string,
    created: Date,
    active: boolean
}

window.addEventListener("load", () => {
    
    console.log("Loaded controller_overview")

    const notifications = new ServerNotifications(NotificationType.ControllerConnection)

    notifications.registerEvent("controller.sessionlist", () => {
        console.log("got controller.sessionlist")
        updateSessionList()
    })

    sessionListDom = new ExtendableList<SessionData>(document.querySelector("#session-list"),  el => {
        let root = document.createElement("div")

        let id = document.createElement("h3")
        id.textContent = el.id

        let created = document.createElement("p")
        created.textContent = el.created.toISOString()

        let active = document.createElement("p")
        active.textContent = el.active ? "ACTIVE" : "TERMINATED"
        
        return root

    }, {emptyMessage: "Keine Sessions erstellt", title: "Sessions"})

    updateSessionList()
})