import { mainComputeShaders } from "./chapters/compute_shaders.js";
import styleText from "./index.scss?inline";


export class CustomApp extends HTMLElement {
    constructor() {
        super();
        // element created

        this.attachShadow({ mode: "open" });

        this.state = {
            canvas: null,
        };
    }

    async connectedCallback() {
        // browser calls this method when the element is added to the document
        // (can be called many times if an element is repeatedly added/removed)

        // Get the source HTML to load
        let templatePath = this.getAttribute("template-path");
        if (!templatePath) return;

        // Get the page
        let request = await fetch(templatePath);
        if (!request.ok) return;

        // Get the HTML
        this.shadowRoot.innerHTML = await request.text();

        // Import styles
        const sheet = new CSSStyleSheet();
        sheet.replaceSync(styleText);
        this.shadowRoot.adoptedStyleSheets = [sheet];

        this.state.canvas = this.shadowRoot.querySelector(".canvas");

        const chapterSelector = this.shadowRoot.querySelector(".chapters");
        if (chapterSelector) {
            chapterSelector.addEventListener("change", (event) => {
                this.renderChapter(event.target.value);
            });
            this.renderChapter(chapterSelector.value);
        }
    }

    async renderChapter(selectedChapter) {
        switch (selectedChapter) {
            case "compute_shaders":
                await mainComputeShaders(this.state.canvas);
                break;
            default:
                await mainComputeShaders(this.state.canvas);
        }
    }

    disconnectedCallback() {
        // browser calls this method when the element is removed from the document
        // (can be called many times if an element is repeatedly added/removed)
    }

    static get observedAttributes() {
        return [/* array of attribute names to monitor for changes */];
    }

    attributeChangedCallback(name, oldValue, newValue) {
        // called when one of attributes listed above is modified
    }

    adoptedCallback() {
        // called when the element is moved to a new document
        // (happens in document.adoptNode, very rarely used)
    }

    // there can be other element methods and properties
}
