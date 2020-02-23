import {getErrorMessage} from './errors'
import {ge, updateToken} from './utils'

window.addEventListener("load", () => {
    console.log("welcome to the admin login page")

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
        if (form.elements["admin-pwd"].value.length > 0) {
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
        validateInput()
        if (blocked) return

        const reqBody = {
            password: form.elements["admin-pwd"].value.trim()
        }



        console.log("trying to connect...")
        console.log(reqBody)
        btn.classList.add("loading")
        const res = await fetch("/api/v1/auth/connect/ctrl", {
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

            console.log(`allowed to control`)
            //window.location.assign(`/ctrl/`)
        } 
        else {
            // no valid login
            console.error("user not allowed to log in")
            const errMsg = await res.text()
            alert(errMsg.length > 0 ? errMsg: "connect failed")
        }
    })
    

    // check if old token is set
    const tokendata = getCurrentTokenData()

    if (tokendata) {
        
        let p = document.createElement("p")
        let reuse = document.createElement("button")
        reuse.textContent = "Wiederverwenden"

        reuse.onclick = () => {
            form.elements["uname"].value = tokendata.username
            form.elements["sid"].value = tokendata.session_id

            validateInput()
        }


        p.textContent = `Letztes Spiel als ${tokendata.username} in Session ${tokendata.session_id}`
        document.querySelector(".last-session").append(p, reuse)
        
    }

})