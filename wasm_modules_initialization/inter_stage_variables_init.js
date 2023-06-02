import init, { Scene } from "../wasm/inter_stage_variables.js";


export async function initInterStageVariables(device, context, gpuTextureFormat) {
    await init();
    const scene = Scene.create(device, context, gpuTextureFormat);
    return scene;    
}
