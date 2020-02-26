import {ServerNotifications, NotificationType} from "./websocket"
import {getCurrentClientTokenString, getCurrentClientTokenData} from "./utils"
import { ExtendableList } from "./ui"

if (getCurrentClientTokenString() == null) {
    
    window.location.assign(`/?error=NoToken`)
}

let playerListDom: ExtendableList<PlayerData>

async function updatePlayerList() {

    const token = getCurrentClientTokenString()
    const tokenParsed = getCurrentClientTokenData()

    if(!token) return


    let req_headers = new Headers()
    req_headers.append("Authorization", `Bearer ${token}`)
    const url = `/api/v1/sessions/${tokenParsed.session_id}/playerlist`

    console.log(url)

    const res = await fetch(url, {
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
    
    const notifications = new ServerNotifications(NotificationType.PlayerConnection)

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