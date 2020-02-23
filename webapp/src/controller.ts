import ServerNotifications from "./websocket"
import {getCurrentAdminTokenString} from "./utils"
import { ExtendableList } from "./ui"

if (getCurrentAdminTokenString() == null) {
    
    window.location.assign(`/ctrl/?error=NoToken`)
}

let playerListDom: ExtendableList<PlayerData>

async function updatePlayerList() {

    const token = getCurrentAdminTokenString()

    if(!token) return


    let req_headers = new Headers()
    req_headers.append("Authorization", `Bearer ${token}`)
    const res = await fetch("/api/v1/session/playerlist", {
        headers: req_headers
    })

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()

        playerListDom.setData(list)
    }
}

interface PlayerData {
    name: string,
    role: object | null
}

window.addEventListener("load", () => {
    
    const notifications = new ServerNotifications()

    notifications.registerEvent("update.playerlist", () => {
        console.log("got update.playerlist")
        updatePlayerList()
    })

    playerListDom = new ExtendableList<PlayerData>(document.querySelector("#playerlist"),  el => {
        let root = document.createElement("p")

        root.textContent = el.name
        return root

    }, {emptyMessage: "Keine Spieler verbunden", title: "Spieler"})

    updatePlayerList()
})