use js_sys::Array;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuTextureDescriptor, GpuExtent3dDict, GpuBindGroupEntry, GpuBindGroupDescriptor, 
    GpuSamplerDescriptor, GpuAddressMode, GpuFilterMode, GpuRenderPipeline, GpuBindGroup, ImageBitmap, 
    GpuImageCopyExternalImage, GpuImageCopyTextureTagged, GpuPrimitiveState, GpuPrimitiveTopology,
    GpuBindGroupLayoutDescriptor, GpuBindGroupLayoutEntry, GpuPipelineLayoutDescriptor, GpuTextureBindingLayout,
    GpuSamplerBindingLayout,
};

use web_sys::gpu_texture_usage::{TEXTURE_BINDING, COPY_DST as TEXTURE_COPY_DST, RENDER_ATTACHMENT};

use web_sys::gpu_shader_stage::FRAGMENT;


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
    bind_groups: Vec<GpuBindGroup>,
    render_pipeline: GpuRenderPipeline,
    render_pipeline_2: GpuRenderPipeline,
}


#[wasm_bindgen]
impl Scene
{
    pub fn create(
        gpu_device: GpuDevice, 
        context: GpuCanvasContext, 
        gpu_texture_format: GpuTextureFormat, 
        image_bitmap: ImageBitmap,
    ) 
        -> Self
    {
        let mut render_shader_module_descriptor = GpuShaderModuleDescriptor::new(
            &include_str!("../shader/render.wgsl"),
        );
        render_shader_module_descriptor.label("our hardcoded textured quad shaders");
        let render_shader_module = gpu_device.create_shader_module(
            &render_shader_module_descriptor,
        );

        let vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let color_target_state = GpuColorTargetState::new(gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new(
            "fragment_main", &render_shader_module, &fragment_state_targets,
        );

        let vertex_state_2 = GpuVertexState::new("vertex_main_2", &render_shader_module);

        let fragment_state_2 = GpuFragmentState::new(
            "fragment_main_2", &render_shader_module, &fragment_state_targets,
        );

        // let render_layout = JsValue::from("auto");
        // let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(
        //     &render_layout, &vertex_state,
        // );
        let sampler_binding_layout = GpuSamplerBindingLayout::new();
        let mut bind_group_layout_0_entry_0 = GpuBindGroupLayoutEntry::new(0, FRAGMENT);
        bind_group_layout_0_entry_0.sampler(&sampler_binding_layout);
        let texture_binding_layout = GpuTextureBindingLayout::new();
        let mut bind_group_layout_0_entry_1 = GpuBindGroupLayoutEntry::new(1, FRAGMENT);
        bind_group_layout_0_entry_1.texture(&texture_binding_layout);
        let bind_group_layout_0_entries = [
            &bind_group_layout_0_entry_0, &bind_group_layout_0_entry_1,
        ].iter().collect::<js_sys::Array>();
        let bind_group_layout_0_descriptor = GpuBindGroupLayoutDescriptor::new(&
            bind_group_layout_0_entries,
        );
        let bind_group_layout_0 = gpu_device.create_bind_group_layout(
            &bind_group_layout_0_descriptor,
        );
        let bind_group_layouts = [&bind_group_layout_0].iter().collect::<js_sys::Array>();
        let pipeline_layout_descriptor = GpuPipelineLayoutDescriptor::new(
            &bind_group_layouts,
        );
        let pipeline_layout = gpu_device.create_pipeline_layout(&pipeline_layout_descriptor);
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(
            &pipeline_layout, &vertex_state,
        );
        render_pipeline_descriptor
            .label("hardcoded textured quad pipeline")
            .fragment(&fragment_state);
        let mut gpu_primitive_state = GpuPrimitiveState::new();
        gpu_primitive_state.topology(GpuPrimitiveTopology::TriangleStrip);
        render_pipeline_descriptor.primitive(&gpu_primitive_state);
        let render_pipeline = gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let mut render_pipeline_2_descriptor = GpuRenderPipelineDescriptor::new(
            &pipeline_layout, &vertex_state_2,
        );
        render_pipeline_2_descriptor
            .label("hardcoded textured quad pipeline 2")
            .fragment(&fragment_state_2);
        render_pipeline_2_descriptor.primitive(&gpu_primitive_state);
        let render_pipeline_2 = gpu_device.create_render_pipeline(&render_pipeline_2_descriptor);

        let texture_descriptor = GpuTextureDescriptor::new(
            GpuTextureFormat::Rgba8unorm,
            &[image_bitmap.width(), image_bitmap.height()].iter().copied().map(JsValue::from).collect::<js_sys::Array>(),
            TEXTURE_BINDING | TEXTURE_COPY_DST | RENDER_ATTACHMENT,
        );
        let texture = gpu_device.create_texture(&texture_descriptor);

        let mut image_copy_external_image = GpuImageCopyExternalImage::new(&image_bitmap);
        image_copy_external_image.flip_y(true);
        let image_copy_texture_tagged = GpuImageCopyTextureTagged::new(&texture);
        let mut gpu_extent_3d_dict = GpuExtent3dDict::new(image_bitmap.width());
        gpu_extent_3d_dict.height(image_bitmap.height());

        gpu_device.queue().copy_external_image_to_texture_with_gpu_extent_3d_dict(
            &image_copy_external_image, 
            &image_copy_texture_tagged, 
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
            let sampler = gpu_device.create_sampler_with_descriptor(&sampler_descriptor);

            let bind_group_0_entry_0 = GpuBindGroupEntry::new(0, &sampler);
            let bind_group_0_entry_1 = GpuBindGroupEntry::new(1, &texture.create_view());
            let bind_group_0_entries = [bind_group_0_entry_0, bind_group_0_entry_1].iter().collect::<Array>();
            // let bind_group_0_descriptor = GpuBindGroupDescriptor::new(
            //     &bind_group_0_entries, &render_pipeline.get_bind_group_layout(0),
            // );
            let bind_group_0_descriptor = GpuBindGroupDescriptor::new(
                &bind_group_0_entries, &bind_group_layout_0,
            );
            let bind_group_0 = gpu_device.create_bind_group(&bind_group_0_descriptor);
            bind_groups.push(bind_group_0);
        }

        Scene 
        {
            gpu_device, context, bind_groups, render_pipeline, render_pipeline_2,
        }
    }


    pub fn render(&mut self, ndx: usize)
    {
        let mut color_attachment = GpuRenderPassColorAttachment::new(
            GpuLoadOp::Clear, GpuStoreOp::Store, &self.context.get_current_texture().create_view(),
        );
        color_attachment.clear_value(&GpuColorDict::new(1.0, 0.3, 0.3, 0.3));
        let color_attachments = [&color_attachment].iter().collect::<js_sys::Array>();
        let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&color_attachments);
        render_pass_descriptor.label("basic canvas render pass");

        let command_encoder = self.gpu_device.create_command_encoder();
        command_encoder.set_label("render quad encoder");

        let render_pass_encoder = command_encoder.begin_render_pass(&render_pass_descriptor);

        render_pass_encoder.set_bind_group(0, Some(&self.bind_groups[ndx]));

        render_pass_encoder.set_pipeline(&self.render_pipeline);
        render_pass_encoder.draw(4);  // call our vertex shader 4 times

        render_pass_encoder.set_pipeline(&self.render_pipeline_2);
        render_pass_encoder.draw(4);  // call our vertex shader 4 times

        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
