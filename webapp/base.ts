

let onSidebarToggle: (bool) => void = b => console.log(`No sidebar... status: ${b}`)




window.addEventListener("load", () => {
    let m_btn: HTMLDivElement = document.querySelector(".menu-btn")
    let sidebar: HTMLElement = document.querySelector("aside")

  

    document.addEventListener("click", e => {
        let elmt = e.target
        let wasOnSideBar = false
        while(elmt) {
            if (elmt == sidebar || elmt == m_btn) {
                wasOnSideBar = true
                break
            }
            elmt = elmt.parentNode
        }
        
        if (!wasOnSideBar) onSidebarToggle(false)
        
    })
    
    m_btn.onclick = () => {
        onSidebarToggle(!m_btn.classList.contains("open"))
    }
    


    if(sidebar) {
        onSidebarToggle = openSideBar => {
            
            m_btn.classList.toggle("open", openSideBar)
            sidebar.classList.toggle("closed", !openSideBar)
        }


        // populate sidebar with basic controls
        let toggleDarkmode = document.createElement("button")

        const setDarkMode = (on: boolean) => {
            localStorage.setItem("darkMode", on.toString())
            toggleDarkmode.textContent = on ? "Light UI" : "Dark UI"
            document.body.classList.toggle("dark", on)
        }

        setDarkMode(localStorage.getItem("darkMode") == "true")
        toggleDarkmode.onclick = () => {
            const toDark = !document.body.classList.contains("dark")
            console.log(`Set darkmode to ${toDark}`)
            setDarkMode(toDark)
            
        }

        sidebar.append(toggleDarkmode)
    }
})