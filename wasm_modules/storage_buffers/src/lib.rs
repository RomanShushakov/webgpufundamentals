use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuBufferDescriptor, 
    GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding,
    HtmlCanvasElement, GpuRenderPipeline, GpuBindGroup, GpuBuffer,

};
use web_sys::gpu_buffer_usage::{COPY_DST, STORAGE};

use js_sys::Float32Array;

use rand::{thread_rng, Rng};


#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(value: &str);
}


fn rand(min: Option<f32>, max: Option<f32>) -> f32
{
    let mut rng = thread_rng();
    if min.is_none() { return rng.gen_range(0.0..1.0); };
    if max.is_none() { return rng.gen_range(0.0..min.unwrap()); };
    rng.gen_range(min.unwrap()..max.unwrap())
}


fn create_circle_vertices(radius: Option<f32>, inner_radius: Option<f32>) -> (Float32Array, u32)
{
    let radius = if let Some(r) = radius { r } else { 1f32 };
    let num_subdivisions = 24;
    let inner_radius = if let Some(i_r) = inner_radius { i_r } else { 0f32 };
    let start_angle = 0f32;
    let end_angle = std::f32::consts::PI * 2.0;

    // 2 triangles per subdivision, 3 verts per tri, 2 values (xy) each.
    let num_vertices = num_subdivisions * 3 * 2;
    let vertex_data = Float32Array::new_with_length(num_subdivisions * 2 * 3 * 2);
   
    let mut offset = 0;
    let mut add_vertex = |x, y| 
        {
            vertex_data.set_index(offset, x);
            offset += 1;
            vertex_data.set_index(offset, y);
            offset += 1;
        };
   
    // 2 vertices per subdivision
    //
    // 0--1 4
    // | / /|
    // |/ / |
    // 2 3--5
    for i in 0..num_subdivisions 
    {
      let angle1 = start_angle + (i + 0) as f32 * (end_angle - start_angle) / num_subdivisions as f32;
      let angle2 = start_angle + (i + 1) as f32 * (end_angle - start_angle) / num_subdivisions as f32;
   
      let c1 = angle1.cos();
      let s1 = angle1.sin();
      let c2 = angle2.cos();
      let s2 = angle2.sin();
   
      // first triangle
      add_vertex(c1 * radius, s1 * radius);
      add_vertex(c2 * radius, s2 * radius);
      add_vertex(c1 * inner_radius, s1 * inner_radius);
   
      // second triangle
      add_vertex(c1 * inner_radius, s1 * inner_radius);
      add_vertex(c2 * radius, s2 * radius);
      add_vertex(c2 * inner_radius, s2 * inner_radius);
    }
   
    (vertex_data, num_vertices)
}


#[wasm_bindgen]
pub struct Scene 
{
    gpu_device: GpuDevice,
    context: GpuCanvasContext,
    object_infos: Vec<f32>,
    num_vertices: u32,
    bind_group_0: GpuBindGroup,
    k_num_objects: u32,
    storage_unit_size: u32,
    changing_storage_buffer_size: u32,
    changing_storage_buffer: GpuBuffer,
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
        let mut render_shader_module_descriptor = GpuShaderModuleDescriptor::new(&include_str!("../shader/render.wgsl"));
        render_shader_module_descriptor.label("triangle shaders with storage buffers");
        let render_shader_module = gpu_device.create_shader_module(&render_shader_module_descriptor);

        let vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let color_target_state = GpuColorTargetState::new(gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &vertex_state);
        render_pipeline_descriptor.label("triangle with storage buffers");
        render_pipeline_descriptor.fragment(&fragment_state);
        let render_pipeline = gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let k_num_objects = 100;
        let mut object_infos = Vec::new();

        let static_storage_unit_size =
            4 * 4 + // color is 4 32bit floats (4bytes each)
            2 * 4 + // offset is 2 32bit floats (4bytes each)
            2 * 4;  // padding

        let storage_unit_size =
            2 * 4;  // scale is 2 32bit floats (4bytes each)

        let static_storage_buffer_size = static_storage_unit_size * k_num_objects;
        let changing_storage_buffer_size = storage_unit_size * k_num_objects;

        let mut static_storage_buffer_descriptor = GpuBufferDescriptor::new(
            static_storage_buffer_size.into(),
            STORAGE | COPY_DST,
        );
        static_storage_buffer_descriptor.label("static storage for objects");
        let static_storage_buffer = gpu_device.create_buffer(&static_storage_buffer_descriptor); 

        let mut changing_storage_buffer_descriptor = GpuBufferDescriptor::new(
            changing_storage_buffer_size.into(),
            STORAGE | COPY_DST,
        );
        changing_storage_buffer_descriptor.label("changing storage for objects");
        let changing_storage_buffer = gpu_device.create_buffer(&changing_storage_buffer_descriptor);

        let k_color_offset = 0u32;
        let k_offset_offset = 4u32;
        
        let static_storage_values = Float32Array::new_with_length(static_storage_buffer_size / 4);

        for i in 0..k_num_objects 
        {
            let static_offset = i * (static_storage_unit_size / 4);

            // These are only set once so set them now
            let color = [rand(None, None), rand(None, None), rand(None, None), 1.0];
            let color_array = Float32Array::new_with_length(color.len() as u32);
            color_array.copy_from(&color);
            static_storage_values.set(&color_array, static_offset + k_color_offset);    // set the color
            let offset = [rand(Some(-0.9), Some(0.9)), rand(Some(-0.9), Some(0.9))];
            let offset_array = Float32Array::new_with_length(offset.len() as u32);
            offset_array.copy_from(&offset);
            static_storage_values.set(&offset_array, static_offset + k_offset_offset);  // set the offset

            object_infos.push(rand(Some(0.2), Some(0.5)));
        }
        gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &static_storage_buffer, 0, &static_storage_values,
        );

        // setup a storage buffer with vertex data
        let (vertex_data, num_vertices) = create_circle_vertices(Some(0.5), Some(0.25));

        let mut vertex_storage_buffer_descriptor = GpuBufferDescriptor::new(
            vertex_data.byte_length().into(),
            STORAGE | COPY_DST,
        );
        vertex_storage_buffer_descriptor.label("storage buffer vertices");
        let vertex_storage_buffer = gpu_device.create_buffer(&vertex_storage_buffer_descriptor);
        gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &vertex_storage_buffer, 0, &vertex_data,
        );

        let bind_group_0_entry_0 = GpuBindGroupEntry::new(0, &GpuBufferBinding::new(&static_storage_buffer));
        let bind_group_0_entry_1 = GpuBindGroupEntry::new(1, &GpuBufferBinding::new(&changing_storage_buffer));
        let bind_group_0_entry_2 = GpuBindGroupEntry::new(2, &GpuBufferBinding::new(&vertex_storage_buffer));
    
        let bind_group_0_entries = [
            bind_group_0_entry_0, bind_group_0_entry_1, bind_group_0_entry_2,
        ].iter().collect::<js_sys::Array>();
        let mut bind_group_0_descriptor = GpuBindGroupDescriptor::new(
            &bind_group_0_entries, &render_pipeline.get_bind_group_layout(0),
        );
        bind_group_0_descriptor.label("bind group for objects");
        let bind_group_0 = gpu_device.create_bind_group(&bind_group_0_descriptor);

        Scene 
        {
            gpu_device, context, object_infos, num_vertices, bind_group_0, k_num_objects, storage_unit_size,
            changing_storage_buffer_size, changing_storage_buffer, render_pipeline,
        }
    }


    pub fn render(&self)
    {
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
        render_pass_encoder.set_pipeline(&self.render_pipeline);

        let canvas = self.context.canvas().dyn_into::<HtmlCanvasElement>().unwrap();
        let aspect = canvas.width() / canvas.height();

        let k_scale_offset = 0u32;

        // a typed array we can use to update the changingStorageBuffer
        let storage_values = Float32Array::new_with_length(self.changing_storage_buffer_size / 4);

        for (ndx, scale) in self.object_infos.iter().enumerate()
        {
            let offset = ndx as u32 * (self.storage_unit_size / 4);

            let scale_vec = [scale / aspect as f32, *scale];
            let scale_array = Float32Array::new_with_length(scale_vec.len() as u32);
            scale_array.copy_from(&scale_vec);
            storage_values.set(&scale_array, offset + k_scale_offset);   // set the scale
        }
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(&self.changing_storage_buffer, 0, &storage_values);

        render_pass_encoder.set_bind_group(0, &self.bind_group_0);
        render_pass_encoder.draw_with_instance_count(self.num_vertices, self.k_num_objects);

        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
