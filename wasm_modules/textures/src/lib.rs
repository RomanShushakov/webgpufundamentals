use std::ops::Deref;

use js_sys::{Float32Array, Array, Uint8Array};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuTextureDescriptor, GpuImageCopyTexture, GpuImageDataLayout, GpuExtent3dDict,
    GpuBindGroupEntry, GpuBindGroupDescriptor, GpuSamplerDescriptor, GpuAddressMode, GpuFilterMode, GpuBufferDescriptor,
    HtmlCanvasElement, GpuBufferBinding, GpuRenderPipeline, GpuBuffer, GpuBindGroup, ContextAttributes2d, ImageData,
};

use web_sys::gpu_texture_usage::{TEXTURE_BINDING, COPY_DST as TEXTURE_COPY_DST};

use web_sys::gpu_buffer_usage::{UNIFORM, COPY_DST as BUFFER_COPY_DST};

#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(value: &str);
}


fn lerp(a: f32, b: f32, t: f32) -> f32
{
    a + (b - a) * t
}


fn mix(a: &[f32], b: &[f32], t: f32) -> Vec<f32>
{
    a.iter().enumerate().map(|(i, v)| lerp(*v, b[i], t)).collect::<Vec<f32>>()
}


fn bilinear_filter(tl: &[f32], tr: &[f32], bl: &[f32], br: &[f32], t1: f32, t2: f32) -> Vec<f32> 
{
    let t = mix(tl, tr, t1);
    let b = mix(bl, br, t1);
    mix(&t, &b, t2)
}


trait MipTrait
{
    fn data(&self) -> Vec<u8>;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
}


#[derive(Clone)]
struct Mip
{
    data: Uint8Array,
    width: f32,
    height: f32,
}


impl MipTrait for Mip
{
    fn data(&self) -> Vec<u8> 
    {
        self.data.to_vec().clone()
    }


    fn width(&self) -> f32
    {
        self.width
    }


    fn height(&self) -> f32 
    {
        self.height
    }
}


impl MipTrait for ImageData
{
    fn data(&self) -> Vec<u8> 
    {
        self.data().deref().clone()
    }


    fn width(&self) -> f32 
    {
        self.width() as f32
    }


    fn height(&self) -> f32 
    {
        self.height() as f32
    }
}


fn create_next_mip_level_rgba8_unorm(mip: Mip) -> Mip
{
    // compute the size of the next mip
    let dst_width = 1f32.max((mip.width / 2.0).floor());    // const dstWidth = Math.max(1, srcWidth / 2 | 0);
    let dst_height = 1f32.max((mip.height / 2.0).floor());  // const dstHeight = Math.max(1, srcHeight / 2 | 0);
    let dst = Uint8Array::new_with_length((dst_width * dst_height * 4.0) as u32);   // const dst = new Uint8Array(dstWidth * dstHeight * 4);

    let get_src_pixel = |x: f32, y: f32| 
        {
            let offset = ((y * mip.width + x) * 4.0) as u32;    // const offset = (y * srcWidth + x) * 4;
            mip.data.subarray(offset, offset + 4)   // return src.subarray(offset, offset + 4);
        };
    
    for y in 0..dst_height as usize // for (let y = 0; y < dstHeight; ++y) {
    {
        for x in 0..dst_width as usize  // for (let x = 0; x < dstWidth; ++x) {
        {
            // compute texcoord of the center of the destination texel
            let u = (x as f32 + 0.5) / dst_width;   // const u = (x + 0.5) / dstWidth;
            let v = (y as f32 + 0.5) / dst_height;   // const v = (y + 0.5) / dstHeight;

            // compute the same texcoord in the source - 0.5 a pixel
            let au = u * mip.width - 0.5;   // const au = (u * srcWidth - 0.5);
            let av = v * mip.height - 0.5;  // const av = (v * srcHeight - 0.5);

            // compute the src top left texel coord (not texcoord)
            let tx = au.floor();    // const tx = au | 0;
            let ty = av.floor();    // const ty = av | 0;

            // compute the mix amounts between pixels
            let t1 = au.fract();    // const t1 = au % 1;
            let t2 = av.fract(); // const t2 = av % 1;

            // get the 4 pixels
            let tl = get_src_pixel(tx, ty); // const tl = getSrcPixel(tx, ty);
            let tr = get_src_pixel(tx + 1.0, ty);   // const tr = getSrcPixel(tx + 1, ty);
            let bl = get_src_pixel(tx, ty + 1.0);   // const bl = getSrcPixel(tx, ty + 1);
            let br = get_src_pixel(tx + 1.0, ty + 1.0); // const br = getSrcPixel(tx + 1, ty + 1);

            // copy the "sampled" result into the dest.
            let dst_offset = (y as f32 * dst_width + x as f32) * 4.0; // const dstOffset = (y * dstWidth + x) * 4;
            dst.set(
                &bilinear_filter(
                    &tl.to_vec().iter().copied().map(|v| v as f32).collect::<Vec<f32>>(),
                    &tr.to_vec().iter().copied().map(|v| v as f32).collect::<Vec<f32>>(), 
                    &bl.to_vec().iter().copied().map(|v| v as f32).collect::<Vec<f32>>(), 
                    &br.to_vec().iter().copied().map(|v| v as f32).collect::<Vec<f32>>(), 
                    t1, 
                    t2,
                ).iter().copied().map(|v| JsValue::from(v as u8)).collect::<Array>(), 
                dst_offset as u32,
            );  // dst.set(bilinearFilter(tl, tr, bl, br, t1, t2), dstOffset);
        }
    }

    Mip { data: dst, width: dst_width, height: dst_height }  // return { data: dst, width: dstWidth, height: dstHeight };
}


fn generate_mips(src: &[u8], src_width: f32) -> Vec<Box<dyn MipTrait>>
{
    let src_height = src.len() as f32 / 4.0 / src_width; // const srcHeight = src.length / 4 / srcWidth;

    let transformed_src = Uint8Array::new(
        &src.iter().copied().map(JsValue::from).collect::<Array>(),
    );

    // populate with first mip level (base level)
    let mut mip = Mip { data: transformed_src, width: src_width, height: src_height };  // let mip = { data: src, width: srcWidth, height: srcHeight, };
    let mut mips: Vec<Box<dyn MipTrait>> = vec![Box::new(mip.clone())];   // const mips = [mip];

    while mip.width > 1.0 || mip.height > 1.0  // while (mip.width > 1 || mip.height > 1) {
    {
        mip = create_next_mip_level_rgba8_unorm(mip); // mip = createNextMipLevelRgba8Unorm(mip);
        mips.push(Box::new(mip.clone())); // mips.push(mip);
    }

    mips
}


fn create_blended_mipmap() -> Vec<Box<dyn MipTrait>> 
{
    let w = [255, 255, 255, 255];
    let r = [255, 0, 0, 255];
    let b = [0, 28, 116, 255];
    let y = [255, 231, 0, 255];
    let g = [58, 181, 75, 255];
    let a = [38, 123, 167, 255];
    let data = [
        w, r, r, r, r, r, r, a, a, r, r, r, r, r, r, w,
        w, w, r, r, r, r, r, a, a, r, r, r, r, r, w, w,
        w, w, w, r, r, r, r, a, a, r, r, r, r, w, w, w,
        w, w, w, w, r, r, r, a, a, r, r, r, w, w, w, w,
        w, w, w, w, w, r, r, a, a, r, r, w, w, w, w, w,
        w, w, w, w, w, w, r, a, a, r, w, w, w, w, w, w,
        w, w, w, w, w, w, w, a, a, w, w, w, w, w, w, w,
        b, b, b, b, b, b, b, b, a, y, y, y, y, y, y, y,
        b, b, b, b, b, b, b, g, y, y, y, y, y, y, y, y,
        w, w, w, w, w, w, w, g, g, w, w, w, w, w, w, w,
        w, w, w, w, w, w, r, g, g, r, w, w, w, w, w, w,
        w, w, w, w, w, r, r, g, g, r, r, w, w, w, w, w,
        w, w, w, w, r, r, r, g, g, r, r, r, w, w, w, w,
        w, w, w, r, r, r, r, g, g, r, r, r, r, w, w, w,
        w, w, r, r, r, r, r, g, g, r, r, r, r, r, w, w,
        w, r, r, r, r, r, r, g, g, r, r, r, r, r, r, w,
    ].into_iter().flatten().collect::<Vec<u8>>();
    generate_mips(&data, 16.0)
}


fn create_checked_mipmap() -> Vec<Box<dyn MipTrait>>
{
    let document = web_sys::window().unwrap().document().unwrap();
    let mut context_options = ContextAttributes2d::new();
    context_options.will_read_frequently(true);
    let ctx = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap()
        .get_context_with_context_options("2d", &context_options.into())
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let levels = [
        (64, "rgb(128,0,255)"),
        (32, "rgb(0,255,0)"),
        (16, "rgb(255,0,0)"),
        (8, "rgb(255,255,0)"),
        (4, "rgb(0,0,255)"),
        (2, "rgb(0,255,255)"),
        (1, "rgb(255,0,255)"),
    ];

    levels.into_iter().enumerate().map(|(i, (size, color))| 
        {
            ctx.canvas().unwrap().set_width(size);
            ctx.canvas().unwrap().set_height(size);
            ctx.set_fill_style(&JsValue::from(if (i & 1) == 1 { "#000" } else { "#fff" }));
            ctx.fill_rect(0.0, 0.0, size as f64, size as f64);
            ctx.set_fill_style(&JsValue::from(color));
            ctx.fill_rect(0.0, 0.0, size as f64 / 2.0, size as f64 / 2.0);
            ctx.fill_rect(size as f64 / 2.0, size as f64 / 2.0, size as f64 / 2.0, size as f64 / 2.0);
            Box::new(ctx.get_image_data(0.0, 0.0, size as f64, size as f64).unwrap()) as Box<dyn MipTrait>
        }).collect::<Vec<Box<dyn MipTrait>>>()
}


#[wasm_bindgen]
pub struct Scene 
{
    gpu_device: GpuDevice,
    context: GpuCanvasContext,
    bind_groups: Vec<GpuBindGroup>,
    uniform_buffer: GpuBuffer,
    uniform_values: Float32Array,
    render_pipeline: GpuRenderPipeline,
}


#[wasm_bindgen]
impl Scene
{
    pub fn create(
        gpu_device: GpuDevice, context: GpuCanvasContext, gpu_texture_format: GpuTextureFormat,
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

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(
            &render_layout, &vertex_state,
        );
        render_pipeline_descriptor
            .label("hardcoded textured quad pipeline")
            .fragment(&fragment_state);
        let render_pipeline = gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let create_texture_with_mips = |mips: Vec<Box<dyn MipTrait>>, label: &str| 
            {
                let mut texture_descriptor = GpuTextureDescriptor::new(
                    GpuTextureFormat::Rgba8unorm,
                    &[mips[0].width(), mips[0].height()].iter().copied().map(JsValue::from).collect::<js_sys::Array>(),
                    TEXTURE_BINDING | TEXTURE_COPY_DST,
                );
                texture_descriptor.label(label);
                texture_descriptor.mip_level_count(mips.len() as u32);

                let texture = gpu_device.create_texture(&texture_descriptor);

                mips.iter().enumerate().for_each(|(mip_level, m)| 
                {
                    let mut gpu_image_copy_texture = GpuImageCopyTexture::new(&texture);
                    gpu_image_copy_texture.mip_level(mip_level as u32);
                    let mut gpu_image_data_layout = GpuImageDataLayout::new();
                    gpu_image_data_layout.bytes_per_row(m.width() as u32 * 4);
                    let mut gpu_extent_3d_dict = GpuExtent3dDict::new(m.width() as u32);
                    gpu_extent_3d_dict.height(m.height() as u32);

                    gpu_device.queue().write_texture_with_u8_array_and_gpu_extent_3d_dict(
                        &gpu_image_copy_texture, 
                        &m.data(), 
                        &gpu_image_data_layout, 
                        &gpu_extent_3d_dict,
                    );
                });

                texture
            };

        let textures = [
            create_texture_with_mips(create_blended_mipmap(), "blended"),
            create_texture_with_mips(create_checked_mipmap(), "checker"),
        ];

        // create a buffer for the uniform values
        let uniform_buffer_size =
            2 * 4 + // scale is 2 32bit floats (4bytes each)
            2 * 4;  // offset is 2 32bit floats (4bytes each)
        let mut buffer_descriptor = GpuBufferDescriptor::new(
            uniform_buffer_size.into(), UNIFORM | BUFFER_COPY_DST,
        );
        buffer_descriptor.label("uniforms for quad");
        let uniform_buffer = gpu_device.create_buffer(&buffer_descriptor);

        // create a typedarray to hold the values for the uniforms in JavaScript
        let uniform_values = Float32Array::new_with_length(uniform_buffer_size / 4);

        let mut bind_groups = Vec::new();
        for i in 0..16
        {
            let mut sampler_descriptor = GpuSamplerDescriptor::new();
            sampler_descriptor
                .address_mode_u(if (i & 1) == 1 { GpuAddressMode::Repeat } else { GpuAddressMode::ClampToEdge })
                .address_mode_v(if (i & 2) == 2 { GpuAddressMode::Repeat } else { GpuAddressMode::ClampToEdge })
                .mag_filter(if (i & 4) == 4 { GpuFilterMode::Linear } else { GpuFilterMode::Nearest })
                .min_filter(if (i & 8) == 8 { GpuFilterMode::Linear } else { GpuFilterMode::Nearest });
            let sampler = gpu_device.create_sampler_with_descriptor(&sampler_descriptor);

            let bind_group_0_entry_0 = GpuBindGroupEntry::new(0, &sampler);
            let bind_group_0_entry_1 = GpuBindGroupEntry::new(1, &textures[0].create_view());
            let bind_group_0_entry_2 = GpuBindGroupEntry::new(2, &GpuBufferBinding::new(&uniform_buffer));
            let bind_group_0_entries = [
                bind_group_0_entry_0, bind_group_0_entry_1, bind_group_0_entry_2,
            ].iter().collect::<Array>();
    
            let bind_group_0_descriptor = GpuBindGroupDescriptor::new(
                &bind_group_0_entries, &render_pipeline.get_bind_group_layout(0),
            );
            let bind_group_0 = gpu_device.create_bind_group(&bind_group_0_descriptor);
            bind_groups.push(bind_group_0);
        }

        Scene 
        {
            gpu_device, context, bind_groups, uniform_buffer, uniform_values, render_pipeline,
        }
    }


    pub fn render(&mut self, ndx: usize, time: f32, scale: f32)
    {
        let bind_group = &self.bind_groups[ndx];

        // offsets to the various uniform values in float32 indices
        let k_scale_offset = 0;
        let k_offset_offset = 2;

        // compute a scale that will draw our 0 to 1 clip space quad
        // 2x2 pixels in the canvas.
        let canvas = self.context.canvas().dyn_into::<HtmlCanvasElement>().unwrap();
        let scale_x = 4.0 / canvas.width() as f32 * scale;
        let scale_y = 4.0 / canvas.height() as f32 * scale;

        self.uniform_values.set(
            &[scale_x, scale_y].iter().copied().map(JsValue::from).collect::<Array>(), 
            k_scale_offset,
        );
        self.uniform_values.set(
            &[f32::sin(time - 0.25) * 0.8, -0.8].iter().copied().map(JsValue::from).collect::<Array>(), 
            k_offset_offset,
        );
    
        // copy the values from JavaScript to the GPU
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &self.uniform_buffer, 0, &self.uniform_values,
        );

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
        render_pass_encoder.set_pipeline(&self.render_pipeline);
        render_pass_encoder.set_bind_group(0, Some(bind_group));
        render_pass_encoder.draw(6);
        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
