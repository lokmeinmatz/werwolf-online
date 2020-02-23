
interface ExtendableListOptions {
    emptyMessage?: string, 
    collapsedLength?: number,
    title: string
}

function setDefaults(elo: ExtendableListOptions) {
    if (!elo.emptyMessage) elo.emptyMessage = "Keine Elemente"
    if (!elo.collapsedLength) elo.collapsedLength = 4
}

export class ExtendableList<T> {

    private isExtended: boolean = false
    private listRoot: HTMLUListElement
    private extendButton: HTMLButtonElement
    private elementGenerator: (el: T) => HTMLElement | null
    private collapsed: boolean = false
    options: ExtendableListOptions

    constructor(root: HTMLDivElement, elementGenrator: (el: T) => HTMLElement | null, options: ExtendableListOptions) {

        let {emptyMessage = "Keine Elemente", collapsedLength = 4} = options

        this.elementGenerator = elementGenrator
        
        root.classList.add("extendable-list")

        const header = document.createElement("h2")
        header.textContent = options.title
        root.appendChild(header)

        setDefaults(options)

        this.options = options
        
        // create list
        this.listRoot = document.createElement("ul")
        root.appendChild(this.listRoot)

        const extendbar = document.createElement("div")
        extendbar.classList.add("extend-bar")

        this.extendButton = document.createElement("button")
        this.extendButton.textContent = "Zeige alle"
        extendbar.appendChild(this.extendButton)
        root.appendChild(extendbar)

        this.extendButton.onclick = () => {
            this.toggleCollapsed()
        }

        this.setEmpty()
    }

    public isCollapsed(): boolean {
        return this.collapsed
    }

    public setCollapsed(collapsed: boolean) {
        if (collapsed == this.collapsed) return
        console.log(`Setting collapsed state: ${collapsed}`)
        this.extendButton.textContent = collapsed ? "Zeige alle" : "Verkleinere Ansicht"


        const itemsStored = this.listRoot.children.length

        for(let i = this.options.collapsedLength; i < itemsStored; i++) {
            this.listRoot.children.item(i).classList.toggle("hidden", collapsed)
        } 
        this.collapsed = collapsed

    }

    public toggleCollapsed() {
        this.setCollapsed(!this.isCollapsed())
    }

    private clear() {
        let last = this.listRoot.lastElementChild
        while(last) {
            this.listRoot.removeChild(last)
            last = this.listRoot.lastElementChild
        }
    }

    setData(data: T[]) {
        this.clear()

        for (let el of data) {
            let generated = this.elementGenerator(el)
            if(generated != null) {
                let li = document.createElement("li")
                li.appendChild(generated)

                this.listRoot.appendChild(li)
            }
        }
    }

    setEmpty() {
        this.clear()

        const emsg = document.createElement("li")
        emsg.textContent = this.options.emptyMessage

        this.listRoot.append(emsg)
    }

}