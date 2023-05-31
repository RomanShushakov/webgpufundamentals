import init, { Scene } from "../wasm/fundamentals.js";


export async function initFundamentals() {
    await init();
    const scene = Scene.create();
    return scene;    
}
