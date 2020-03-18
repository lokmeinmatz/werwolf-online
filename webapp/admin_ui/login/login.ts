import {getErrorMessage} from '../../src/errors'
import {getCurrentTokenString, updateToken} from '../../src/utils'



    // check if old token is set
if(getCurrentTokenString() != null) {

}

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



        console.log("trying to connect as admin...")
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
            window.location.assign(`/ctrl/overview`)
        } 
        else {
            // no valid login
            console.error("user not allowed to log in")
            const errMsg = await res.text()
            alert(errMsg.length > 0 ? errMsg: "connect failed")
        }
    })
    



})