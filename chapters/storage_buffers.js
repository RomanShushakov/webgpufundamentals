import { initStorageBuffers } from "../wasm_modules_initialization/storage_buffers_init.js";


function fail(msg) {
    alert(msg);
}

export async function mainStorageBuffers(canvas) {
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

        // 'reason' will be "destroyed" if we intentionally destroy the device.
        if (info.reason !== "destroyed") {
            // try again
            mainStorageBuffers(canvas);
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

    const scene = await initStorageBuffers(device, context, gpuTextureFormat);

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
