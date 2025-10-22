import init, { Scene } from "../wasm/compute_shader_basics.js";

export async function initComputeShaderBasics(device) {
  await init();
  const scene = Scene.create(device);
  return scene;
}
