use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuTextureDescriptor, GpuImageCopyTexture, GpuImageDataLayout, GpuExtent3dDict,
    GpuBindGroupEntry, GpuBindGroupDescriptor, GpuSamplerDescriptor, GpuAddressMode, GpuFilterMode,
};

use web_sys::gpu_texture_usage::
{
    TEXTURE_BINDING, COPY_DST,
};

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


    pub fn render(&self, ndx: usize)
    {
        let mut render_shader_module_descriptor = GpuShaderModuleDescriptor::new(&include_str!("../shader/render.wgsl"));
        render_shader_module_descriptor.label("our hardcoded textured quad shaders");
        let render_shader_module = self.gpu_device.create_shader_module(&render_shader_module_descriptor);

        let vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let color_target_state = GpuColorTargetState::new(self.gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &vertex_state);
        render_pipeline_descriptor
            .label("hardcoded textured quad pipeline")
            .fragment(&fragment_state);
        let render_pipeline = self.gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let k_texture_width = 5;
        let k_texture_height = 7;

        let r = [255, 0, 0, 255];  // red
        let y = [255, 255, 0, 255];  // yellow
        let b = [0, 0, 255, 255];  // blue

        let texture_data = [
            b, r, r, r, r,
            r, y, y, y, r,
            r, y, r, r, r,
            r, y, y, r, r,
            r, y, r, r, r,
            r, y, r, r, r,
            r, r, r, r, r,
        ].into_iter().flatten().collect::<Vec<u8>>();

        let mut texture_descriptor = GpuTextureDescriptor::new(
            GpuTextureFormat::Rgba8unorm,
            &[k_texture_width, k_texture_height].iter().copied().map(JsValue::from).collect::<js_sys::Array>(),
            TEXTURE_BINDING | COPY_DST,
        );
        texture_descriptor.label("yellow F on red");

        let texture = self.gpu_device.create_texture(&texture_descriptor);

        let gpu_image_copy_texture = GpuImageCopyTexture::new(&texture);
        let mut gpu_image_data_layout = GpuImageDataLayout::new();
        gpu_image_data_layout.bytes_per_row(k_texture_width * 4);
        let mut gpu_extent_3d_dict = GpuExtent3dDict::new(k_texture_width);
        gpu_extent_3d_dict.height(k_texture_height);

        self.gpu_device.queue().write_texture_with_u8_array_and_gpu_extent_3d_dict(
            &gpu_image_copy_texture, 
            &texture_data, 
            &gpu_image_data_layout, 
            &gpu_extent_3d_dict,
        );

        let mut bind_groups = Vec::new();
        for i in 0..8
        {
            let mut sampler_descriptor = GpuSamplerDescriptor::new();
            sampler_descriptor
                .address_mode_u(if (i & 1) == 1 { GpuAddressMode::Repeat } else { GpuAddressMode::ClampToEdge })
                .address_mode_v(if (i & 2) == 2 { GpuAddressMode::Repeat } else { GpuAddressMode::ClampToEdge })
                .mag_filter(if (i & 4) == 4 { GpuFilterMode::Linear } else { GpuFilterMode::Nearest });
            let sampler = self.gpu_device.create_sampler_with_descriptor(&sampler_descriptor);

            let bind_group_0_entry_0 = GpuBindGroupEntry::new(0, &sampler);
            let bind_group_0_entry_1 = GpuBindGroupEntry::new(1, &texture.create_view());
            let bind_group_0_entries = [bind_group_0_entry_0, bind_group_0_entry_1].iter().collect::<js_sys::Array>();
    
            let bind_group_0_descriptor = GpuBindGroupDescriptor::new(
                &bind_group_0_entries, &render_pipeline.get_bind_group_layout(0),
            );
            let bind_group_0 = self.gpu_device.create_bind_group(&bind_group_0_descriptor);
            bind_groups.push(bind_group_0);
        }

        let mut color_attachment = GpuRenderPassColorAttachment::new(
            GpuLoadOp::Clear, GpuStoreOp::Store, &self.context.get_current_texture().create_view(),
        );
        color_attachment.clear_value(&GpuColorDict::new(1.0, 0.3, 0.3, 0.3));
        let color_attachments = [color_attachment].iter().collect::<js_sys::Array>();
        let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
        render_pass_descriptor.label("basic canvas render pass");

        let command_encoder = self.gpu_device.create_command_encoder();
        command_encoder.set_label("render quad encoder");

        let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass_encoder.set_pipeline(&render_pipeline);
        render_pass_encoder.set_bind_group(0, &bind_groups[ndx]);
        render_pass_encoder.draw(6);
        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
