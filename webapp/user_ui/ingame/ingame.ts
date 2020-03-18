import {ServerNotifications, NotificationType} from "/src/websocket"
import {getCurrentTokenString, getCurrentPlayerTokenData, apiFetch} from "../../src/utils"
import { ExtendableList } from "../../src/ui"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/?error=NoToken`)
}

let playerListDom: ExtendableList<PlayerData>

async function updatePlayerList() {

    const token = getCurrentTokenString()
    const tokenParsed = getCurrentPlayerTokenData()

    if(!token) return


    const url = `/sessions/${tokenParsed.session_id}/playerlist`


    const res = await apiFetch(url)

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