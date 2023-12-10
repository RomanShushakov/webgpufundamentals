import init, { Scene } from "../wasm/loading_images.js";


export async function initLoadingImages(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
