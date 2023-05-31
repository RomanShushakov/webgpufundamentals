import init, { Scene } from "../wasm/fundamentals.js";


export async function initFundamentals(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
