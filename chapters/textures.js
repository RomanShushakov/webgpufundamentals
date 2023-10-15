import { initTextures } from "../wasm_modules_initialization/textures_init.js";
import * as dat from "dat.gui";

const settings = {
    addressModeU: "repeat",
    addressModeV: "repeat",
    magFilter: "linear",
};

function findIndex(settings) {
    return (settings.addressModeU === 'repeat' ? 1 : 0) + 
        (settings.addressModeV === 'repeat' ? 2 : 0) +
        (settings.magFilter === 'linear' ? 4 : 0);
}

let gui;
function addGUI(scene) {
    const addressOptions = ["repeat", "clamp-to-edge"];
    const filterOptions = ["nearest", "linear"];

    gui = new dat.GUI();
    Object.assign(gui.domElement.style, { position: "absolute", left: "0", top: "2rem" });

    gui.add(settings, "addressModeU", addressOptions).onChange(() => scene.render(findIndex(settings)));
    gui.add(settings, "addressModeV", addressOptions).onChange(() => scene.render(findIndex(settings)));
    gui.add(settings, "magFilter", filterOptions).onChange(() => scene.render(findIndex(settings)));
}

export function destroyTexturesGUI() {
    gui?.destroy();
    gui = undefined;
}

export async function mainTextures(canvas) {
    if (!navigator.gpu) {
        fail('this browser does not support WebGPU');
        return;
    }

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
        fail('this browser supports webgpu but it appears disabled');
        return;
    }

    const device = await adapter?.requestDevice();
    device.lost.then((info) => {
        console.error(`WebGPU device was lost: ${info.message}`);

        // 'reason' will be 'destroyed' if we intentionally destroy the device.
        if (info.reason !== 'destroyed') {
            // try again
            mainTextures(canvas);
        }
    });

    if (!canvas) {
        console.log("There are no canvas provided")
        return;
    }

    canvas.style.imageRendering = "pixelated";
    canvas.style.imageRendering = "crisp-edges";
    
    const context = canvas.getContext("webgpu");

    const gpuTextureFormat = navigator.gpu.getPreferredCanvasFormat();
    context.configure({
        device,
        format: gpuTextureFormat,
    });

    const scene = await initTextures(device, context, gpuTextureFormat);

    addGUI(scene);

    const observer = new ResizeObserver(entries => {
        for (const entry of entries) {
            const canvas = entry.target;
            const width = entry.contentBoxSize[0].inlineSize / 64 | 0;
            const height = entry.contentBoxSize[0].blockSize / 64 | 0;
            canvas.width = Math.min(width, device.limits.maxTextureDimension2D);
            canvas.height = Math.min(height, device.limits.maxTextureDimension2D);
            // re-render
            scene.render(findIndex(settings));
        }
    });
    observer.observe(canvas);
}
