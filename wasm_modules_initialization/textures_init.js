import init, { Scene } from "../wasm/textures.js";


export async function initTextures(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
