import {ServerNotifications, NotificationType} from "../../src/websocket"
import {getCurrentTokenString, getCurrentPlayerTokenData} from "../../src/utils"
import { ExtendableList } from "../../src/ui"
import * as api from "../../src/api"

if (getCurrentTokenString() == null) {
    
    window.location.assign(`/?error=NoToken`)
}

let playerListDom: ExtendableList<api.PlayerData>

async function updatePlayerList() {

    const tokenParsed = getCurrentPlayerTokenData()
    try {
        const list = await api.getPlayerList(tokenParsed.session_id)
        playerListDom.setData(list)
    } catch (error) {
        console.error(error)
    }
    
}



window.addEventListener("load", () => {
    
    const notifications = new ServerNotifications(NotificationType.PlayerConnection)

    notifications.registerEvent("update.playerlist", () => {
        console.log("got update.playerlist")
        updatePlayerList()
    })

    playerListDom = new ExtendableList<api.PlayerData>(document.querySelector("#playerlist"),  el => {
        let root = document.createElement("p")

        root.textContent = el.name
        return root

    }, {emptyMessage: "Keine Spieler verbunden", title: "Spieler"})

    updatePlayerList()
})