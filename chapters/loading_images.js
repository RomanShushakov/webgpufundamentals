import { initLoadingImages } from "../wasm_modules_initialization/loading_images_init.js";
import * as dat from "dat.gui";

const settings = {
    addressModeU: "repeat",
    addressModeV: "repeat",
    magFilter: "linear",
};

const addressOptions = ["repeat", "clamp-to-edge"];
const filterOptions = ["nearest", "linear"];

let gui;
function addGUI(fnc) {
    gui = new dat.GUI();
    Object.assign(gui.domElement.style, { position: "absolute", left: "0", top: "2rem" });

    gui.add(settings, "addressModeU", addressOptions).onChange(fnc);
    gui.add(settings, "addressModeV", addressOptions).onChange(fnc);
    gui.add(settings, "magFilter", filterOptions).onChange(fnc);
}

export function destroyLoadingImagesGUI() {
    gui?.destroy();
    gui = undefined;
}

function fail(msg) {
    alert(msg);
}

async function loadImageBitmap(url) {
    const res = await fetch(url);
    const blob = await res.blob();
    return await createImageBitmap(blob, { colorSpaceConversion: "none" });
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

    const url = "./assets/images/f-texture.png";
    const imageBitmap = await loadImageBitmap(url);

    const scene = await initLoadingImages(device, context, gpuTextureFormat, imageBitmap);

    function render() {
        const ndx = (settings.addressModeU === 'repeat' ? 1 : 0) +
            (settings.addressModeV === 'repeat' ? 2 : 0) +
            (settings.magFilter === 'linear' ? 4 : 0);
        scene.render(ndx);
        console.log("Rendered");
    };

    addGUI(render);

    const observer = new ResizeObserver(entries => {
        for (const entry of entries) {
            const canvas = entry.target;
            const width = entry.contentBoxSize[0].inlineSize;
            const height = entry.contentBoxSize[0].blockSize;
            canvas.width = Math.max(1, Math.min(width, device.limits.maxTextureDimension2D));
            canvas.height = Math.max(1, Math.min(height, device.limits.maxTextureDimension2D));
            render();
        }
    });
    observer.observe(canvas);
}
