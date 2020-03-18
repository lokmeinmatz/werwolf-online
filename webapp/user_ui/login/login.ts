import {getErrorMessage} from '/src/errors'
import {getCurrentPlayerTokenData, updateToken, apiFetch} from '/src/utils'

window.addEventListener("load", async () => {
    console.log("welcome to the start page")

    const urlParams = new URLSearchParams(window.location.search)

    let refErrorMsg = urlParams.get("error")
    if(refErrorMsg) {
        alert(getErrorMessage(refErrorMsg))
    }

    const form: HTMLFormElement = document.querySelector("form.connect")
    
    let btn = document.querySelector(".connect-btn")
    btn.classList.add("blocked")
    let blocked = true
    function validateInput() {
        if (form.elements["uname"].value.length > 0 && form.elements["sid"].value.length > 0) {
            blocked = false
            btn.classList.remove("blocked")
        }
        else {
            blocked = true,
            btn.classList.add("blocked")
        }
    }

    validateInput()
    
    form.addEventListener("input", validateInput)

    form.addEventListener("submit", async event => {
        event.preventDefault()

        if (blocked) return

        const reqBody = {
            username: form.elements["uname"].value.trim(),
            session_id: form.elements["sid"].value.trim()
        }



        console.log("trying to connect...")
        console.log(reqBody)
        btn.classList.add("loading")
        const res = await fetch("/api/v1/auth/connect/client", {
            method: "POST",
            body: JSON.stringify(reqBody),
            headers: {
                "Content-Type": "application/json"
            }
        })
        btn.classList.remove("loading")
        if (res.status == 200) {
            // store jwt
            const jwt = await res.text()

            updateToken(jwt)

            console.log(`allowed to join`)
            window.location.assign(`/game/`)
        } 
        else {
            // no valid login
            console.error("user not allowed to log in")
            const errMsg = await res.text()
            alert(errMsg.length > 0 ? errMsg: "connect failed")
        }
    })
    

    // check if old token is set
    const tokendata = getCurrentPlayerTokenData()

    if (tokendata) {

        // test if session still active
        const res = await apiFetch(`/sessions/${tokendata.session_id}`)

        if (!res.ok) {
            console.log("No session data for last used session")
            return
        }
        const sInfo: {active: boolean} = await res.json()
        
        if (!sInfo.active) {
            console.log("Last used session is not active")
            return
        }
        
        let p = document.createElement("p")
        let retry = document.createElement("a")
        retry.textContent = "Session wieder beitreteten"
        retry.href = "/game"



        p.textContent = `Letztes Spiel als ${tokendata.user_name} in Session ${tokendata.session_id}`
        document.querySelector(".last-session").append(p, retry)
        
    }

})