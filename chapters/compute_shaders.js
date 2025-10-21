import { initComputeShaders } from "../wasm_modules_initialization/compute_shaders_init.js";

export async function mainComputeShaders(canvas) {
  const adapter = await navigator.gpu?.requestAdapter();
  const device = await adapter?.requestDevice();
  if (!device) {
    console.log("need a browser that supports WebGPU");
    return;
  }

  if (!canvas) {
    console.log("There are no canvas provided");
    return;
  }

  const context = canvas.getContext("webgpu");

  const gpuTextureFormat = navigator.gpu.getPreferredCanvasFormat();
  context.configure({
    device,
    format: gpuTextureFormat,
  });

  const scene = await initComputeShaders(device, context, gpuTextureFormat);

  const input = new Float32Array([1, 3, 5, 7]);
  const output = await scene.compute(input);

  console.log("Input:", input);
  console.log("Output:", output);
}
