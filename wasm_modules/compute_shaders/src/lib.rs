use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use wasm_bindgen_futures::JsFuture;

use web_sys::
{
    GpuDevice, GpuShaderModuleDescriptor, GpuComputePipelineDescriptor, GpuProgrammableStage, GpuBufferDescriptor,
    GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding, GpuComputePassDescriptor,
};
use web_sys::gpu_buffer_usage::{COPY_SRC, COPY_DST, STORAGE, MAP_READ};

use web_sys::gpu_map_mode::READ;

use js_sys::Float32Array;


#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(value: &str);
}

fn compose_shader_code(replace_from: &str, replace_to: &str) -> String
{
    include_str!("../shader/compute.wgsl").replace(replace_from, replace_to)
}


#[wasm_bindgen]
pub struct Scene 
{
    gpu_device: GpuDevice,
}


#[wasm_bindgen]
impl Scene
{
    pub fn create(
        gpu_device: GpuDevice,
    ) 
        -> Self
    {
        Scene 
        {
            gpu_device,
        }
    }


    pub async fn compute(&self, input: &[f32]) -> Result<Float32Array, JsValue>
    {
        let dispatch_count = [4, 3, 2];
        let work_group_size = [2, 3, 4];

        // multiply all elements of an array
        let array_prod = |arr: &[i32]| arr.iter().fold(1, |acc, x| acc * x);

        let num_threads_per_work_group = array_prod(&work_group_size);

        log(&num_threads_per_work_group.to_string());

        let command_encoder = self.gpu_device.create_command_encoder();
        command_encoder.set_label("Our command encoder");

        let compute_shader_code = compose_shader_code("${work_group_size}", "1");
        let compute_shader_module_descriptor = GpuShaderModuleDescriptor::new(&compute_shader_code);
        compute_shader_module_descriptor.set_label("Doubling compute module");
        let compute_shader_module = self.gpu_device.create_shader_module(&compute_shader_module_descriptor);

        let compute_stage = GpuProgrammableStage::new(&compute_shader_module);

        let compute_layout = JsValue::from("auto");
        let compute_pipeline_descriptor = GpuComputePipelineDescriptor::new(&compute_layout, &compute_stage);
        compute_pipeline_descriptor.set_label("Doubling compute pipeline");
        let compute_pipeline = self.gpu_device.create_compute_pipeline(&compute_pipeline_descriptor);

        let input_array = Float32Array::from(input);

        let compute_input_buffer_descriptor = GpuBufferDescriptor::new(
            input_array.byte_length().into(),
            STORAGE | COPY_DST | COPY_SRC,
        );
        compute_input_buffer_descriptor.set_label("Compute input buffer");
        let compute_input_buffer = self.gpu_device.create_buffer(&compute_input_buffer_descriptor).unwrap();
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&compute_input_buffer, 0, &input_array)?;

        let compute_result_buffer_descriptor = GpuBufferDescriptor::new(
            input_array.byte_length().into(),
            MAP_READ | COPY_DST,
        );
        compute_result_buffer_descriptor.set_label("Compute result buffer");
        let compute_result_buffer = self.gpu_device.create_buffer(&compute_result_buffer_descriptor).unwrap();

        let bind_group_entry_resource = GpuBufferBinding::new(&compute_input_buffer);
        let bind_group_entry = GpuBindGroupEntry::new(0, &bind_group_entry_resource);
        let bind_group_entries = [bind_group_entry].iter().collect::<js_sys::Array>();
        let compute_bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group_descriptor = GpuBindGroupDescriptor::new(&bind_group_entries, &compute_bind_group_layout);
        let bind_group = self.gpu_device.create_bind_group(&bind_group_descriptor);

        let compute_pass_descriptor = GpuComputePassDescriptor::new();
        compute_pass_descriptor.set_label("Doubling compute pass");
        let compute_pass = command_encoder.begin_compute_pass_with_descriptor(&compute_pass_descriptor);
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, Some(&bind_group));
        compute_pass.dispatch_workgroups(input_array.length());
        compute_pass.end();

        command_encoder.copy_buffer_to_buffer_with_u32_and_u32_and_f64(
            &compute_input_buffer, 0, &compute_result_buffer, 0, compute_result_buffer.size(),
        )?;


        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());


        JsFuture::from(compute_result_buffer.map_async(READ)).await?;
        let result_buffer = compute_result_buffer.get_mapped_range().unwrap();
        let output = Float32Array::new(&result_buffer.slice(0));
        compute_result_buffer.unmap();

        Ok(output)
    }
}
