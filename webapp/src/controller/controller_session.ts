import {ServerNotifications, NotificationType} from "../websocket"
import {getCurrentTokenString, apiFetch} from "../utils"
import { ExtendableList } from "../ui"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/ctrl/?error=NoToken`)
}

let playerListDom: ExtendableList<PlayerData>

const currentSessionID: string = window.location.pathname.split("/")[3]

async function updatePlayerList() {

    
    const url = `/sessions/${currentSessionID}/playerlist`
    
    const res = await apiFetch(url)

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()
        console.log(list)
        playerListDom.setData(list)
    }
}

interface PlayerData {
    name: string,
    joined: Date,
    state: string,
    role: string
}

window.addEventListener("load", () => {
    
    console.log("Loaded controller_overview")

    const notifications = new ServerNotifications(NotificationType.ControllerConnection)

    notifications.registerEvent("update.playerlist", () => {
        updatePlayerList()
    })

    playerListDom = new ExtendableList<PlayerData>(document.querySelector("#player-list"),  el => {
        let root = document.createElement("div")

        let name = document.createElement("h3")
        name.textContent = el.name

        
        let joined = document.createElement("p")
        const created_data = new Date(el.joined)
        joined.textContent = created_data.toISOString()
        
        let state = document.createElement("p")
        state.textContent = el.state

        let role = document.createElement("p")
        role.textContent = `Spieler: ${el.role}`
        
        root.appendChild(name)
        root.appendChild(joined)
        root.appendChild(state)
        root.appendChild(role)
        return root
        

    }, {emptyMessage: "Keine Spieler in dieser Session", title: "Spieler"})

    updatePlayerList()
})