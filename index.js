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


    const scene = await initFundamentals();
    scene.greeting("wasm");
}

await main();
