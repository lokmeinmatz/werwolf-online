import ServerNotifications from "./websocket"

if (localStorage.getItem("token") == null) {
    
    window.location.assign(`/?error=NoToken`)
}

window.addEventListener("load", () => {
    const notifications = new ServerNotifications()


    notifications.registerEvent("update.playerlist", () => {
        console.log("got update.playerlist")
    })
})