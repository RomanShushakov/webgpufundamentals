import init, { Scene } from "../wasm/vertex_buffers.js";


export async function initVertexBuffers(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
