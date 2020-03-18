import {ServerNotifications, NotificationType} from "../../src/websocket"
import {getCurrentTokenString, apiFetch} from "../../src/utils"
import { ExtendableList } from "../../src/ui"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/ctrl/?error=NoToken`)
}

let playerListDom: ExtendableList<PlayerData>

const currentSessionID: string = window.location.pathname.split("/")[3]

async function updatePlayerList() {

    
    const url = `/sessions/${currentSessionID}/playerlist`
    
    const res = await apiFetch(url)

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list: [PlayerData] = await res.json()
        
        for (let e of list) {
            e.joined = new Date(e.joined as unknown as number * 1000)
        }
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
        joined.textContent = el.joined.toLocaleString("DE-de")
        
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