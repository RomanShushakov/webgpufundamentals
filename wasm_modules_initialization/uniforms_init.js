import init, { Scene } from "../wasm/uniforms.js";


export async function initUniforms(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
