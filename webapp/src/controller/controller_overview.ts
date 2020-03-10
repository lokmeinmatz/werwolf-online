import {ServerNotifications, NotificationType} from "../websocket"
import {getCurrentTokenString, apiFetch} from "../utils"
import { ExtendableList } from "../ui"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/ctrl/?error=NoToken`)
}

let sessionListDom: ExtendableList<SessionData>

async function updateSessionList() {

    const token = getCurrentTokenString()

    if(!token) return
    

    const res = await apiFetch("/sessions/")

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()
        console.log(list)
        sessionListDom.setData(list)
    }
}

interface SessionData {
    id: string,
    created: Date,
    active: boolean,
    player_count: number
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

        let id = document.createElement("a")
        id.href = `/ctrl/session/${el.id}`
        id.textContent = el.id

        
        let created = document.createElement("p")
        const created_data = new Date(el.created)
        created.textContent = created_data.toISOString()
        
        let active = document.createElement("p")
        active.textContent = el.active ? "ACTIVE" : "TERMINATED"

        let player_count = document.createElement("p")
        player_count.textContent = `Spieler: ${el.player_count}`
        
        root.appendChild(id)
        root.appendChild(created)
        root.appendChild(active)
        root.appendChild(player_count)

        return root
        

    }, {emptyMessage: "Keine Sessions erstellt", title: "Sessions"})

    updateSessionList()
})