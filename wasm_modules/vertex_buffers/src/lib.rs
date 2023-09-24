use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuBufferDescriptor,
    HtmlCanvasElement, GpuVertexBufferLayout, GpuVertexAttribute, GpuVertexFormat, GpuVertexStepMode,

};
use web_sys::gpu_buffer_usage::{COPY_DST, VERTEX};

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
        let mut render_shader_module_descriptor = GpuShaderModuleDescriptor::new(
            &include_str!("../shader/render.wgsl"),
        );
        render_shader_module_descriptor.label("triangle shaders with vertex buffers");
        let render_shader_module = self.gpu_device.create_shader_module(
            &render_shader_module_descriptor,
        );

        let mut vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let vertex_position_format = GpuVertexFormat::Float32x2;
        let vertex_position_buffer_attribute = GpuVertexAttribute::new(
            vertex_position_format, 0f64, 0,    // position
        );
        let vertex_position_buffer_attributes = [
            vertex_position_buffer_attribute,
        ].iter().collect::<js_sys::Array>();
        let vertex_position_buffer_layout = GpuVertexBufferLayout::new(
            2f64 * 4f64, &vertex_position_buffer_attributes,    // 2 floats, 4 bytes each
        );

        let vertex_color_format = GpuVertexFormat::Float32x4;
        let vertex_color_buffer_attribute = GpuVertexAttribute::new(
            vertex_color_format, 0f64, 1,   // color
        );
        let vertex_offset_format = GpuVertexFormat::Float32x2;
        let vertex_offset_buffer_attribute = GpuVertexAttribute::new(
            vertex_offset_format, 16f64, 2,   // offset
        );
        let vertex_color_offset_buffer_attributes = [
            vertex_color_buffer_attribute, vertex_offset_buffer_attribute,
        ].iter().collect::<js_sys::Array>();
        let mut vertex_color_offset_buffer_layout = GpuVertexBufferLayout::new(
            6f64 * 4f64, &vertex_color_offset_buffer_attributes,    // 6 floats, 4 bytes each
        );
        vertex_color_offset_buffer_layout.step_mode(GpuVertexStepMode::Instance);

        let vertex_scale_format = GpuVertexFormat::Float32x2;
        let vertex_scale_buffer_attribute = GpuVertexAttribute::new(
            vertex_scale_format, 0f64, 3,    // scale
        );
        let vertex_scale_buffer_attributes = [
            vertex_scale_buffer_attribute,
        ].iter().collect::<js_sys::Array>();
        let mut vertex_scale_buffer_layout = GpuVertexBufferLayout::new(
            2f64 * 4f64, &vertex_scale_buffer_attributes,    // 2 floats, 4 bytes each
        );
        vertex_scale_buffer_layout.step_mode(GpuVertexStepMode::Instance);

        let vertex_buffers = [
            vertex_position_buffer_layout, vertex_color_offset_buffer_layout, vertex_scale_buffer_layout,
        ].iter().collect::<js_sys::Array>();
        vertex_state.buffers(&vertex_buffers);

        let color_target_state = GpuColorTargetState::new(self.gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &vertex_state);
        render_pipeline_descriptor.label("triangle with vertex buffers");
        render_pipeline_descriptor.fragment(&fragment_state);
        let render_pipeline = self.gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let k_num_objects = 100;
        let mut object_infos = Vec::new();

        let static_unit_size =
            4 * 4 + // color is 4 32bit floats (4bytes each)
            2 * 4;  // offset is 2 32bit floats (4bytes each)

        let changing_unit_size =
            2 * 4;  // scale is 2 32bit floats (4bytes each)

        let static_vertex_buffer_size = static_unit_size * k_num_objects;
        let changing_vertex_buffer_size = changing_unit_size * k_num_objects;

        let mut static_vertex_buffer_descriptor = GpuBufferDescriptor::new(
            static_vertex_buffer_size.into(),
            VERTEX | COPY_DST,
        );
        static_vertex_buffer_descriptor.label("static storage for objects");
        let static_vertex_buffer = self.gpu_device.create_buffer(&static_vertex_buffer_descriptor); 

        let mut changing_vertex_buffer_descriptor = GpuBufferDescriptor::new(
            changing_vertex_buffer_size.into(),
            VERTEX | COPY_DST,
        );
        changing_vertex_buffer_descriptor.label("changing storage for objects");
        let changing_vertex_buffer = self.gpu_device.create_buffer(&changing_vertex_buffer_descriptor);

        let k_color_offset = 0u32;
        let k_offset_offset = 4u32;
        let k_scale_offset = 0u32;
        
        let static_vertex_values = Float32Array::new_with_length(static_vertex_buffer_size / 4);

        for i in 0..k_num_objects 
        {
            let static_offset = i * (static_unit_size / 4);

            // These are only set once so set them now
            let color = [rand(None, None), rand(None, None), rand(None, None), 1.0];
            let color_array = Float32Array::new_with_length(color.len() as u32);
            color_array.copy_from(&color);
            static_vertex_values.set(&color_array, static_offset + k_color_offset);    // set the color
            let offset = [rand(Some(-0.9), Some(0.9)), rand(Some(-0.9), Some(0.9))];
            let offset_array = Float32Array::new_with_length(offset.len() as u32);
            offset_array.copy_from(&offset);
            static_vertex_values.set(&offset_array, static_offset + k_offset_offset);  // set the offset

            object_infos.push(rand(Some(0.2), Some(0.5)));
        }
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &static_vertex_buffer, 0, &static_vertex_values,
        );

        // a typed array we can use to update the changingStorageBuffer
        let changing_vertex_values = Float32Array::new_with_length(changing_vertex_buffer_size / 4);

        // setup a storage buffer with vertex data
        let (vertex_data, num_vertices) = create_circle_vertices(Some(0.5), Some(0.25));

        let mut vertex_buffer_descriptor = GpuBufferDescriptor::new(
            vertex_data.byte_length().into(),
            VERTEX | COPY_DST,
        );
        vertex_buffer_descriptor.label("vertex buffer vertices");
        let vertex_buffer = self.gpu_device.create_buffer(&vertex_buffer_descriptor);
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &vertex_buffer, 0, &vertex_data,
        );

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
        render_pass_encoder.set_vertex_buffer(0, &vertex_buffer);
        render_pass_encoder.set_vertex_buffer(1, &static_vertex_buffer);
        render_pass_encoder.set_vertex_buffer(2, &changing_vertex_buffer);

        let canvas = self.context.canvas().dyn_into::<HtmlCanvasElement>().unwrap();
        let aspect = canvas.width() / canvas.height();

        for (ndx, scale) in object_infos.iter().enumerate()
        {
            let offset = ndx as u32 * (changing_unit_size / 4);

            let scale_vec = [scale / aspect as f32, *scale];
            let scale_array = Float32Array::new_with_length(scale_vec.len() as u32);
            scale_array.copy_from(&scale_vec);
            changing_vertex_values.set(&scale_array, offset + k_scale_offset);   // set the scale
        }
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &changing_vertex_buffer, 0, &changing_vertex_values,
        );

        render_pass_encoder.draw_with_instance_count(num_vertices, k_num_objects);

        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
