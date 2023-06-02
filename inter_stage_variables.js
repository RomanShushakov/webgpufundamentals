import { initInterStageVariables } from "./wasm_modules_initialization/inter_stage_variables_init.js";


export async function mainInterStageVariables() {
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

    const scene = await initInterStageVariables(device, context, gpuTextureFormat);


    const observer = new ResizeObserver(entries => {
        for (const entry of entries) {
            const canvas = entry.target;
            const width = entry.contentBoxSize[0].inlineSize;
            const height = entry.contentBoxSize[0].blockSize;
            canvas.width = Math.min(width, device.limits.maxTextureDimension2D);
            canvas.height = Math.min(height, device.limits.maxTextureDimension2D);
            // re-render
            scene.render();
        }
    });
    observer.observe(canvas);
}
