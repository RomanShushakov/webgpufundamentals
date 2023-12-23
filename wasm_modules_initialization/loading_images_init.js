import init, { Scene } from "../wasm/loading_images.js";


export async function initLoadingImages(device, context, gpuTextureFormat, imageBitmap) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat, imageBitmap);
    return scene;    
}
