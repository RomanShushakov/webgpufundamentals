import { initFundamentals } from "./wasm_modules_initialization/fundamentals_init.js";


async function main() {
    const adapter = await navigator.gpu?.requestAdapter();
    const device = await adapter?.requestDevice();
    if (!device) {
        console.log("need a browser that supports WebGPU");
        return;
    }

    const canvas = document.getElementById("canvas");
    if (!canvas) {
        console.log("There are no canvas provided")
        return;
    }

    const context = canvas.getContext("webgpu");

    const gpuTextureFormat = navigator.gpu.getPreferredCanvasFormat();
    context.configure({
        device,
        format: gpuTextureFormat,
    });

    const scene = await initFundamentals(device, context, gpuTextureFormat);
    
    scene.render();
}

await main();
