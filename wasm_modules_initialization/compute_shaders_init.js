import init, { Scene } from "../wasm/compute_shaders.js";

export async function initComputeShaders(device, context, gpuTextureFormat) {
  await init();
  const scene = Scene.create(device, context, gpuTextureFormat);
  return scene;
}
