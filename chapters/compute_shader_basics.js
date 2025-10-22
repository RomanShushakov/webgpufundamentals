import { initComputeShaderBasics } from "../wasm_modules_initialization/compute_shader_basics_init.js";

function log(...args) {
  const elem = document.createElement("pre");
  elem.textContent = args.join(" ");
  document.body.appendChild(elem);
}

export async function mainComputeShaderBasics() {
  const adapter = await navigator.gpu?.requestAdapter();
  const device = await adapter?.requestDevice();
  if (!device) {
    console.log("need a browser that supports WebGPU");
    return;
  }

  const scene = await initComputeShaderBasics(device);

  const outputObject = await scene.compute();

  const numResults = outputObject["num_results"];
  const numThreadsPerWorkgroup = outputObject["num_threads_per_workgroup"];
  const workgroup = outputObject["workgroup_read_output"];
  const local = outputObject["local_read_output"];
  const global = outputObject["global_read_output"];

  const get3 = (arr, i) => {
    const off = i * 4;
    return `${arr[off]}, ${arr[off + 1]}, ${arr[off + 2]}`;
  };

  for (let i = 0; i < numResults; ++i) {
    if (i % numThreadsPerWorkgroup === 0) {
      log(`\
---------------------------------------
global                 local     global   dispatch: ${
        i / numThreadsPerWorkgroup
      }
invoc.    workgroup    invoc.    invoc.
index     id           id        id
---------------------------------------`);
    }
    log(
      `${i.toString().padStart(3)}:      ${get3(workgroup, i)}      ${get3(
        local,
        i
      )}   ${get3(global, i)}`
    );
  }
}
