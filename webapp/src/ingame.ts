import ServerNotifications from "./websocket"
import {getCurrentTokenString} from "./utils"
import { ExtendableList } from "./ui"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/?error=NoToken`)
}

let playerListDom

async function updatePlayerList() {

    const token = getCurrentTokenString()

    if(!token) return


    let req_headers = new Headers()
    req_headers.append("Authorization", `Bearer ${token}`)
    const res = await fetch("/api/v1/session/playerlist", {
        headers: req_headers
    })

    if (res.status == 200 && res.headers.get("Content-Type") == "application/json") {
        let list = await res.json()

        console.log(list)
    }
}

window.addEventListener("load", () => {
    const notifications = new ServerNotifications()

    notifications.registerEvent("update.playerlist", () => {
        console.log("got update.playerlist")
        updatePlayerList()
    })

    playerListDom = new ExtendableList(document.querySelector("#playerlist"), "Keine Spieler verbunden")

    updatePlayerList()
})