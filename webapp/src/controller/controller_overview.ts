import {ServerNotifications, NotificationType} from "../websocket"
import {getCurrentTokenString} from "../utils"
import { ExtendableList } from "../ui"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/ctrl/?error=NoToken`)
}

let sessionListDom: ExtendableList<SessionData>

async function updateSessionList() {

    const token = getCurrentTokenString()

    if(!token) return

    console.log("requesting session list")

    let req_headers = new Headers()
    req_headers.append("Authorization", `Bearer ${token}`)
    const res = await fetch("/api/v1/sessions/", {
        headers: req_headers
    })

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()
        console.log(list)
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
        id.textContent = el

        root.appendChild(id)

        return root
        let created = document.createElement("p")
        created.textContent = el.created.toISOString()

        let active = document.createElement("p")
        active.textContent = el.active ? "ACTIVE" : "TERMINATED"
        

    }, {emptyMessage: "Keine Sessions erstellt", title: "Sessions"})

    updateSessionList()
})