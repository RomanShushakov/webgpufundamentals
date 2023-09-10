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

use rand::{thread_rng, Rng};


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
        let mut render_shader_module_descriptor = GpuShaderModuleDescriptor::new(&include_str!("../shader/render.wgsl"));
        render_shader_module_descriptor.label("triangle shaders with uniforms");
        let render_shader_module = self.gpu_device.create_shader_module(&render_shader_module_descriptor);

        let vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let color_target_state = GpuColorTargetState::new(self.gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &vertex_state);
        render_pipeline_descriptor.label("triangle with uniforms");
        render_pipeline_descriptor.fragment(&fragment_state);
        let render_pipeline = self.gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let uniform_buffer_size =
            4 * 4 +     // color is 4 32bit floats (4bytes each)
            2 * 4 +     // scale is 2 32bit floats (4bytes each)
            2 * 4;      // offset is 2 32bit floats (4bytes each)

        let rand = |min: Option<f32>, max: Option<f32>| 
        {
            let mut rng = thread_rng();
            if min.is_none() { return rng.gen_range(0.0..1.0); };
            if max.is_none() { return rng.gen_range(0.0..min.unwrap()); };
            rng.gen_range(min.unwrap()..max.unwrap())
        };

        let k_color_offset: u32 = 0;
        let k_scale_offset: u32 = 4;
        let k_offset_offset: u32 = 6;

        let k_num_objects = 100;
        let mut object_infos = Vec::new();

        for i in 0..k_num_objects
        {
            let mut uniform_buffer_descriptor = GpuBufferDescriptor::new(
                uniform_buffer_size.into(),
                UNIFORM | COPY_DST,
            );
            uniform_buffer_descriptor.label(&format!("uniforms for obj: {}", i));
            let uniform_buffer = self.gpu_device.create_buffer(&uniform_buffer_descriptor);   
            let uniform_values = Float32Array::new_with_length(uniform_buffer_size / 4);

            let color = [rand(None, None), rand(None, None), rand(None, None), 1.0];
            let color_array = Float32Array::new_with_length(color.len() as u32);
            color_array.copy_from(&color);
            uniform_values.set(&color_array, k_color_offset);       // set the color

            let offset = [rand(Some(-0.9), Some(0.9)), rand(Some(-0.9), Some(0.9))];
            let offset_array = Float32Array::new_with_length(offset.len() as u32);
            offset_array.copy_from(&offset);
            uniform_values.set(&offset_array, k_offset_offset);     // set the offset

            let bind_group_0_entry_resource = GpuBufferBinding::new(&uniform_buffer);
            let bind_group_0_entry = GpuBindGroupEntry::new(0, &bind_group_0_entry_resource);
        
            let bind_group_0_entries = [bind_group_0_entry].iter().collect::<js_sys::Array>();
            let mut bind_group_0_descriptor = GpuBindGroupDescriptor::new(
                &bind_group_0_entries, &render_pipeline.get_bind_group_layout(0),
            );
            bind_group_0_descriptor.label(&format!("bind group 0 for obj: {}", i));
            let bind_group_0 = self.gpu_device.create_bind_group(&bind_group_0_descriptor);

            object_infos.push((rand(Some(0.2), Some(0.5)), uniform_buffer, uniform_values, bind_group_0));
        } 

        let mut color_attachment = GpuRenderPassColorAttachment::new(
            GpuLoadOp::Clear, GpuStoreOp::Store, &self.context.get_current_texture().create_view(),
        );
        color_attachment.clear_value(&GpuColorDict::new(1.0, 0.3, 0.3, 0.3));
        let color_attachments = [color_attachment].iter().collect::<js_sys::Array>();
        let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
        render_pass_descriptor.label("basic canvas render pass");

        let command_encoder = self.gpu_device.create_command_encoder();
        command_encoder.set_label("command encoder");

        let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass_encoder.set_pipeline(&render_pipeline);

        let canvas = self.context.canvas().dyn_into::<HtmlCanvasElement>().unwrap();
        let aspect = canvas.width() / canvas.height();

        for (scale, uniform_buffer, uniform_values, bind_group_0) in object_infos 
        {
            let scale = [scale / aspect as f32, scale];
            let scale_array = Float32Array::new_with_length(scale.len() as u32);
            scale_array.copy_from(&scale);
            uniform_values.set(&scale_array, k_scale_offset);       // set the scale

            self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&uniform_buffer, 0, &uniform_values);

            render_pass_encoder.set_bind_group(0, &bind_group_0);
            render_pass_encoder.draw(3);
        }

        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
