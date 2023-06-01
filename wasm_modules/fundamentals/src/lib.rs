use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use wasm_bindgen_futures::{JsFuture, spawn_local};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuComputePipelineDescriptor, GpuProgrammableStage, GpuBufferDescriptor, 
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


#[wasm_bindgen]
pub struct Scene 
{
    gpu_device: GpuDevice,
    context: GpuCanvasContext,
    gpu_texture_format: GpuTextureFormat,
}


#[wasm_bindgen]
impl Scene
{
    pub fn create(
        gpu_device: GpuDevice, context: GpuCanvasContext, gpu_texture_format: GpuTextureFormat,
    ) 
        -> Self
    {
        Scene 
        {
            gpu_device, context, gpu_texture_format,
        }
    }


    pub fn render(&self)
    {
        let command_encoder = self.gpu_device.create_command_encoder();
        command_encoder.set_label("Our command encoder");


        let mut render_shader_module_descriptor = GpuShaderModuleDescriptor::new(&include_str!("../shader/render.wgsl"));
        render_shader_module_descriptor.label("Our hardcoded red triangle shaders");
        let render_shader_module = self.gpu_device.create_shader_module(&render_shader_module_descriptor);

        let vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let color_target_state = GpuColorTargetState::new(self.gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &vertex_state);
        render_pipeline_descriptor.label("Our hardcoded red triangle pipeline");
        render_pipeline_descriptor.fragment(&fragment_state);
        let render_pipeline = self.gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let mut color_attachment = GpuRenderPassColorAttachment::new(
            GpuLoadOp::Clear, GpuStoreOp::Store, &self.context.get_current_texture().create_view(),
        );
        color_attachment.clear_value(&GpuColorDict::new(1.0, 1.0, 0.0, 0.0));
        let color_attachments = [color_attachment].iter().collect::<js_sys::Array>();
        let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
        render_pass_descriptor.label("Our basic canvas render pass");

        let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass_encoder.set_pipeline(&render_pipeline);
        render_pass_encoder.draw(3);
        render_pass_encoder.end();


        let mut compute_shader_module_descriptor = GpuShaderModuleDescriptor::new(&include_str!("../shader/compute.wgsl"));
        compute_shader_module_descriptor.label("Doubling compute module");
        let compute_shader_module = self.gpu_device.create_shader_module(&compute_shader_module_descriptor);

        let compute_stage = GpuProgrammableStage::new("compute_main", &compute_shader_module);

        let compute_layout = JsValue::from("auto");
        let mut compute_pipeline_descriptor = GpuComputePipelineDescriptor::new(&compute_layout, &compute_stage);
        compute_pipeline_descriptor.label("Doubling compute pipeline");
        let compute_pipeline = self.gpu_device.create_compute_pipeline(&compute_pipeline_descriptor);

        let input_array = Float32Array::from([1.0f32, 3.0, 5.0].as_slice());

        let mut compute_input_buffer_descriptor = GpuBufferDescriptor::new(
            input_array.byte_length().into(),
            STORAGE | COPY_DST | COPY_SRC,
        );
        compute_input_buffer_descriptor.label("Compute input buffer");
        let compute_input_buffer = self.gpu_device.create_buffer(&compute_input_buffer_descriptor);
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&compute_input_buffer, 0, &input_array);

        let mut compute_result_buffer_descriptor = GpuBufferDescriptor::new(
            input_array.byte_length().into(),
            MAP_READ | COPY_DST,
        );
        compute_result_buffer_descriptor.label("Compute result buffer");
        let compute_result_buffer = self.gpu_device.create_buffer(&compute_result_buffer_descriptor);

        let bind_group_entry_resource = GpuBufferBinding::new(&compute_input_buffer);
        let bind_group_entry = GpuBindGroupEntry::new(0, &bind_group_entry_resource);
        let bind_group_entries = [bind_group_entry].iter().collect::<js_sys::Array>();
        let compute_bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group_descriptor = GpuBindGroupDescriptor::new(&bind_group_entries, &compute_bind_group_layout);
        let bind_group = self.gpu_device.create_bind_group(&bind_group_descriptor);

        let mut compute_pass_descriptor = GpuComputePassDescriptor::new();
        compute_pass_descriptor.label("Doubling compute pass");
        let compute_pass = command_encoder.begin_compute_pass_with_descriptor(&compute_pass_descriptor);
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group);
        compute_pass.dispatch_workgroups(input_array.length());
        compute_pass.end();

        command_encoder.copy_buffer_to_buffer_with_u32_and_u32_and_f64(
            &compute_input_buffer, 0, &compute_result_buffer, 0, compute_result_buffer.size(),
        );


        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());


        log(&format!("{:?}", input_array.to_vec()));

        spawn_local(async move
            {
                JsFuture::from(compute_result_buffer.map_async(READ)).await.unwrap();
                let result_buffer = compute_result_buffer.get_mapped_range();
                log(&format!("{:?}", Float32Array::new(&result_buffer).to_vec()));
                compute_result_buffer.unmap();
            }
        );
    }
}
