import { mainFundamentals } from "./chapters/fundamentals.js";
import { mainInterStageVariables } from "./chapters/inter_stage_variables.js";
import { mainUniforms } from "./chapters/uniforms.js";
import { mainStorageBuffers } from "./chapters/storage_buffers.js";
import { mainVertexBuffers } from "./chapters/vertex_buffers.js";
import { mainTextures, destroyTexturesGUI, cancelTexturesAnimation } from "./chapters/textures.js";
import styleText from "./styles.scss?inline";


export class CustomApp extends HTMLElement {
    constructor() {
        super();
        // element created

        const sheet = new CSSStyleSheet();
        sheet.replaceSync(styleText);

        this.state = {
            canvas: null,
        };

        this.attachShadow({ mode: "open" });

        this.shadowRoot.adoptedStyleSheets = [sheet];
        this.shadowRoot.innerHTML = 
        /*html*/
        `
        <div class="wrapper">
            <div class="select-chapters-container">
                <label class="label" for="chapters">Choose a chapter:</label>
                <select name="chapters" class="chapters">
                    <option value="textures">Textures</option>
                    <option value="vertex_buffers">Vertex buffers</option>
                    <option value="storage_buffers">Storage buffers</option>
                    <option value="uniforms">Uniforms</option>
                    <option value="inter_stage_variables">Inter-stage variables</option>
                    <option value="fundamentals">Fundamentals</option>
                </select>
            </div>
            <canvas class="canvas"></canvas>
        </div>
        `;

        this.state.canvas = this.shadowRoot.querySelector(".canvas");

        const chapterSelector = this.shadowRoot.querySelector(".chapters");
        if (chapterSelector) {
            chapterSelector.addEventListener("change", (event) => {
                this.renderChapter(event.target.value);
            });

        }
    }

    connectedCallback() {
        // browser calls this method when the element is added to the document
        // (can be called many times if an element is repeatedly added/removed)

        const selectedChapter = this.shadowRoot.querySelector(".chapters")?.value;
        if (selectedChapter) {
            this.renderChapter(selectedChapter);
        }
    }

    async renderChapter(selectedChapter) {
        switch (selectedChapter) {
            case "fundamentals":
                await mainFundamentals(this.state.canvas);
                destroyTexturesGUI();
                cancelTexturesAnimation();
                break;
            case "inter_stage_variables":
                await mainInterStageVariables(this.state.canvas);
                destroyTexturesGUI();
                cancelTexturesAnimation();
                break;
            case "uniforms":
                await mainUniforms(this.state.canvas);
                destroyTexturesGUI();
                cancelTexturesAnimation();
                break;
            case "storage_buffers":
                await mainStorageBuffers(this.state.canvas);
                destroyTexturesGUI();
                cancelTexturesAnimation();
                break;
            case "vertex_buffers":
                await mainVertexBuffers(this.state.canvas);
                destroyTexturesGUI();
                cancelTexturesAnimation();
                break;
            case "textures":
                await mainTextures(this.state.canvas);
                break;
            default:
                await mainFundamentals(this.state.canvas);
                destroyTexturesGUI();
                cancelTexturesAnimation();
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
