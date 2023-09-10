import init, { Scene } from "../wasm/storage_buffers.js";


export async function initStorageBuffers(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
