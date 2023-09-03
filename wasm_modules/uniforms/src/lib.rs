use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuBufferDescriptor, 
    GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding,
    HtmlCanvasElement,

};
use web_sys::gpu_buffer_usage::{COPY_DST, UNIFORM};

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
        render_shader_module_descriptor.label("triangle shaders with uniforms");
        let render_shader_module = self.gpu_device.create_shader_module(&render_shader_module_descriptor);

        let render_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let color_target_state = GpuColorTargetState::new(self.gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &render_state);
        render_pipeline_descriptor.label("hardcoded checkerboard triangle pipeline");
        render_pipeline_descriptor.fragment(&fragment_state);
        let render_pipeline = self.gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let uniform_buffer_size =
            4 * 4 +     // color is 4 32bit floats (4bytes each)
            2 * 4 +     // scale is 2 32bit floats (4bytes each)
            2 * 4;      // offset is 2 32bit floats (4bytes each)
        let mut uniform_buffer_descriptor = GpuBufferDescriptor::new(
            uniform_buffer_size.into(),
            UNIFORM | COPY_DST,
        );
        uniform_buffer_descriptor.label("uniforms");
        let uniform_buffer = self.gpu_device.create_buffer(&uniform_buffer_descriptor);

        let uniform_values = Float32Array::new_with_length(uniform_buffer_size / 4);

        let k_color_offset: u32 = 0;
        let k_scale_offset: u32 = 4;
        let k_offset_offset: u32 = 6;

        let color = [0.0, 1.0, 0.0, 1.0];
        let color_array = Float32Array::new_with_length(color.len() as u32);
        color_array.copy_from(&color);
        uniform_values.set(&color_array, k_color_offset);       // set the color
        let offset = [-0.5, -0.25];
        let offset_array = Float32Array::new_with_length(offset.len() as u32);
        offset_array.copy_from(&offset);
        uniform_values.set(&offset_array, k_offset_offset);     // set the offset

        let bind_group_entry_0_resource = GpuBufferBinding::new(&uniform_buffer);
        let bind_group_entry_0 = GpuBindGroupEntry::new(0, &bind_group_entry_0_resource);
    
        let bind_group_entries = [bind_group_entry_0].iter().collect::<js_sys::Array>();
        let mut bind_group_descriptor = GpuBindGroupDescriptor::new(
            &bind_group_entries, &render_pipeline.get_bind_group_layout(0),
        );
        bind_group_descriptor.label("bind group 0");
        let bind_group = self.gpu_device.create_bind_group(&bind_group_descriptor);

        let canvas = self.context.canvas().dyn_into::<HtmlCanvasElement>().unwrap();
        let aspect = canvas.width() / canvas.height();
        let scale = [0.5 / aspect as f32, 0.5];
        let scale_array = Float32Array::new_with_length(scale.len() as u32);
        scale_array.copy_from(&scale);
        uniform_values.set(&scale_array, k_scale_offset);       // set the scale

        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_values);

        let mut color_attachment = GpuRenderPassColorAttachment::new(
            GpuLoadOp::Clear, GpuStoreOp::Store, &self.context.get_current_texture().create_view(),
        );
        color_attachment.clear_value(&GpuColorDict::new(0.3, 0.3, 0.3, 1.0));
        let color_attachments = [color_attachment].iter().collect::<js_sys::Array>();
        let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
        render_pass_descriptor.label("basic canvas render pass");

        let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass_encoder.set_pipeline(&render_pipeline);
        render_pass_encoder.set_bind_group(0, &bind_group);
        render_pass_encoder.draw(3);
        render_pass_encoder.end();


        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
