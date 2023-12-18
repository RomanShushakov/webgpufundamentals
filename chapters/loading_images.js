import { initLoadingImages } from "../wasm_modules_initialization/loading_images_init.js";


function fail(msg) {
    alert(msg);
}

export async function mainLoadingImages(canvas) {
    if (!navigator.gpu) {
        fail("this browser does not support WebGPU");
        return;
    }

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
        fail("this browser supports webgpu but it appears disabled");
        return;
    }

    const device = await adapter?.requestDevice();
    device.lost.then((info) => {
        console.error(`WebGPU device was lost: ${info.message}`);

        // "reason" will be "destroyed" if we intentionally destroy the device.
        if (info.reason !== "destroyed") {
            // try again
            mainLoadingImages(canvas);
        }
    });

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

    const scene = await initLoadingImages(device, context, gpuTextureFormat);

    let texNdx = 0;

    const observer = new ResizeObserver(entries => {
        for (const entry of entries) {
            const canvas = entry.target;
            const width = entry.contentBoxSize[0].inlineSize;
            const height = entry.contentBoxSize[0].blockSize;
            canvas.width = Math.max(1, Math.min(width, device.limits.maxTextureDimension2D));
            canvas.height = Math.max(1, Math.min(height, device.limits.maxTextureDimension2D));
            scene.render(texNdx);
        }
    });
    observer.observe(canvas);

    canvas.addEventListener("click", () => {
        texNdx = (texNdx + 1) % 2;
        scene.render(texNdx);
    });
}
