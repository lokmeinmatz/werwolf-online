


export class ExtendableList {

    private isExtended: boolean = false
    private listRoot: HTMLUListElement
    private extendButton: HTMLButtonElement
    emptyMessage: string

    constructor(root: HTMLDivElement, emptyMessage: string) {
        
        root.classList.add("extendable-list")
        this.emptyMessage = emptyMessage
        
        // create list
        this.listRoot = document.createElement("ul")
        root.appendChild(this.listRoot)

        const extendbar = document.createElement("div")
        extendbar.classList.add("extend-bar")

        this.extendButton = document.createElement("button")
        extendbar.appendChild(this.extendButton)
        root.appendChild(extendbar)

        this.setEmpty()
    }

    setEmpty() {
        let last = this.listRoot.lastElementChild
        while(last) {
            this.listRoot.removeChild(last)
            last = this.listRoot.lastElementChild
        }

        const emsg = document.createElement("li")
        emsg.textContent = this.emptyMessage

        this.listRoot.append(emsg)
    }

}