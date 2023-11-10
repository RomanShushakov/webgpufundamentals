import { initTextures } from "../wasm_modules_initialization/textures_init.js";
import * as dat from "dat.gui";

const settings = {
    addressModeU: "repeat",
    addressModeV: "repeat",
    magFilter: "linear",
    minFilter: "linear",
    scale: 1,
};

const addressOptions = ["repeat", "clamp-to-edge"];
const filterOptions = ["nearest", "linear"];

let gui;
function addGUI() {
    gui = new dat.GUI();
    Object.assign(gui.domElement.style, { position: "absolute", left: "0", top: "2rem" });

    gui.add(settings, "addressModeU", addressOptions);
    gui.add(settings, "addressModeV", addressOptions);
    gui.add(settings, "magFilter", filterOptions);
    gui.add(settings, "minFilter", filterOptions);
    gui.add(settings, "scale", 0.5, 6);
}

function fail(msg) {
    alert(msg);
}

export function destroyTexturesGUI() {
    gui?.destroy();
    gui = undefined;
}

let animation;

export function cancelTexturesAnimation() {
    cancelAnimationFrame(animation);
}

export async function mainTextures(canvas) {
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

    addGUI();

    function run(time) {
        time *= 0.001;
        const ndx = (settings.addressModeU === "repeat" ? 1 : 0) +
            (settings.addressModeV === "repeat" ? 2 : 0) +
            (settings.magFilter === "linear" ? 4 : 0) +
            (settings.minFilter === "linear" ? 8 : 0);

        scene.render(ndx, time, settings.scale);
        animation = requestAnimationFrame(run);
    }
    animation = requestAnimationFrame(run);

    const observer = new ResizeObserver(entries => {
        for (const entry of entries) {
            const canvas = entry.target;
            const width = entry.contentBoxSize[0].inlineSize / 64 | 0;
            const height = entry.contentBoxSize[0].blockSize / 64 | 0;
            canvas.width = Math.min(width, device.limits.maxTextureDimension2D);
            canvas.height = Math.min(height, device.limits.maxTextureDimension2D);
        }
    });
    observer.observe(canvas);
}
