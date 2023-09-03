use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use wasm_bindgen_futures::JsFuture;

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
        render_shader_module_descriptor.label("Our hardcoded rgb triangle shaders");
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


        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
