use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};

use web_sys::
{
    GpuDevice, GpuCanvasContext, GpuTextureFormat, GpuShaderModuleDescriptor, GpuVertexState, GpuColorTargetState, 
    GpuFragmentState, GpuRenderPipelineDescriptor, GpuRenderPassColorAttachment, GpuLoadOp, GpuStoreOp, GpuColorDict, 
    GpuRenderPassDescriptor, GpuBufferDescriptor, HtmlCanvasElement, GpuVertexBufferLayout, GpuVertexAttribute, 
    GpuVertexFormat, GpuVertexStepMode, GpuIndexFormat, GpuRenderPipeline, GpuBuffer,

};
use web_sys::gpu_buffer_usage::{COPY_DST, VERTEX, INDEX};

use js_sys::{Float32Array, Uint8Array, Uint32Array};

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


fn create_circle_vertices(radius: Option<f32>, inner_radius: Option<f32>) -> (Float32Array, Uint32Array, u32)
{
    let radius = if let Some(r) = radius { r } else { 1f32 };
    let num_subdivisions = 24;
    let inner_radius = if let Some(i_r) = inner_radius { i_r } else { 0f32 };
    let start_angle = 0f32;
    let end_angle = std::f32::consts::PI * 2.0;

    // 2 vertices at each subdivision, + 1 to wrap around the circle.
    let num_vertices = (num_subdivisions + 1) * 2;
    
    // 2 32-bit values for position (xy) and 1 32-bit value for color (rgb_)
    // The 32-bit color value will be written/read as 4 8-bit values
    let vertex_data = Float32Array::new_with_length(num_vertices * (2 + 1));
    let color_data = Uint8Array::new(&vertex_data.buffer());
   
    let mut offset = 0;
    let mut color_offset = 8;

    let mut add_vertex = |x, y, r, g, b| 
        {
            vertex_data.set_index(offset, x);
            offset += 1;
            vertex_data.set_index(offset, y);
            offset += 1;
            offset += 1;    // skip the color

            color_data.set_index(color_offset, (r * 255.0) as u8);
            color_offset += 1;
            color_data.set_index(color_offset, (g * 255.0) as u8);
            color_offset += 1;
            color_data.set_index(color_offset, (b * 255.0) as u8);
            color_offset += 1;
            color_offset += 9;  // skip extra byte and the position
        };

    let inner_color = [1.0, 1.0, 1.0];
    let outer_color = [0.1, 0.1, 0.1];

    // 2 vertices per subdivision
    //
    // 0  2  4  6  8 ...
    //
    // 1  3  5  7  9 ...
    for i in 0..=num_subdivisions
    {
        let angle = start_angle + (i + 0) as f32 * (end_angle - start_angle) / num_subdivisions as f32;
 
        let c1 = angle.cos();
        let s1 = angle.sin();
 
        add_vertex(c1 * radius, s1 * radius, outer_color[0], outer_color[1], outer_color[2]);
        add_vertex(c1 * inner_radius, s1 * inner_radius, inner_color[0], inner_color[1], inner_color[2]);
    }
 
    let index_data = Uint32Array::new_with_length(num_subdivisions * 6);
    let mut ndx = 0;
 
    // 0---2---4---...
    // | //| //|
    // |// |// |//
    // 1---3-- 5---...
    for i in 0..num_subdivisions
    {
        let ndx_offset = i * 2;
 
        // first triangle
        index_data.set_index(ndx, ndx_offset);
        ndx += 1;
        index_data.set_index(ndx, ndx_offset + 1);
        ndx += 1;
        index_data.set_index(ndx, ndx_offset + 2);
        ndx += 1;
 
        // second triangle
        index_data.set_index(ndx, ndx_offset + 2);
        ndx += 1;
        index_data.set_index(ndx, ndx_offset + 1);
        ndx += 1;
        index_data.set_index(ndx, ndx_offset + 3);
        ndx += 1;
    }

    let num_indexes = index_data.length();
   
    (vertex_data, index_data, num_indexes)
}


#[wasm_bindgen]
pub struct Scene 
{
    gpu_device: GpuDevice,
    context: GpuCanvasContext,
    vertex_buffer: GpuBuffer,
    static_vertex_buffer: GpuBuffer,
    changing_vertex_buffer: GpuBuffer,
    index_buffer: GpuBuffer,
    render_pipeline: GpuRenderPipeline,
    object_infos: Vec<f32>,
    changing_unit_size: u32,
    changing_vertex_values: Float32Array,
    num_indexes: u32,
    k_num_objects: u32,
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
        render_shader_module_descriptor.label("triangle shaders with vertex buffers");
        let render_shader_module = gpu_device.create_shader_module(
            &render_shader_module_descriptor,
        );

        let mut vertex_state = GpuVertexState::new("vertex_main", &render_shader_module);

        let vertex_position_format = GpuVertexFormat::Float32x2;
        let vertex_position_buffer_attribute = GpuVertexAttribute::new(
            vertex_position_format, 0f64, 0,    // position
        );
        let per_vertex_color_format = GpuVertexFormat::Unorm8x4;
        let per_vertex_color_buffer_attribute = GpuVertexAttribute::new(
            per_vertex_color_format, 8f64, 4,    // per vertex color
        );
        let vertex_position_buffer_attributes = [
            vertex_position_buffer_attribute, per_vertex_color_buffer_attribute,
        ].iter().collect::<js_sys::Array>();
        let vertex_position_buffer_layout = GpuVertexBufferLayout::new(
            2f64 * 4f64 + 4f64, &vertex_position_buffer_attributes,    // 2 floats, 4 bytes each + 4 bytes
        );

        let vertex_color_format = GpuVertexFormat::Unorm8x4;
        let vertex_color_buffer_attribute = GpuVertexAttribute::new(
            vertex_color_format, 0f64, 1,   // color
        );
        let vertex_offset_format = GpuVertexFormat::Float32x2;
        let vertex_offset_buffer_attribute = GpuVertexAttribute::new(
            vertex_offset_format, 4f64, 2,   // offset
        );
        let vertex_color_offset_buffer_attributes = [
            vertex_color_buffer_attribute, vertex_offset_buffer_attribute,
        ].iter().collect::<js_sys::Array>();
        let mut vertex_color_offset_buffer_layout = GpuVertexBufferLayout::new(
            4f64 + 2f64 * 4f64, &vertex_color_offset_buffer_attributes,    // 4 bytes + 2 floats, 4 bytes each
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

        let color_target_state = GpuColorTargetState::new(gpu_texture_format);
        let fragment_state_targets = [color_target_state].iter().collect::<js_sys::Array>();
        let fragment_state = GpuFragmentState::new("fragment_main", &render_shader_module, &fragment_state_targets);

        let render_layout = JsValue::from("auto");
        let mut render_pipeline_descriptor = GpuRenderPipelineDescriptor::new(&render_layout, &vertex_state);
        render_pipeline_descriptor.label("triangle with vertex buffers");
        render_pipeline_descriptor.fragment(&fragment_state);
        let render_pipeline = gpu_device.create_render_pipeline(&render_pipeline_descriptor);

        let k_num_objects = 100;
        let mut object_infos = Vec::new();

        // create 2 vertex buffers
        let static_unit_size =
            4 +     // color is 4 bytes
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
        let static_vertex_buffer = gpu_device.create_buffer(&static_vertex_buffer_descriptor); 

        let mut changing_vertex_buffer_descriptor = GpuBufferDescriptor::new(
            changing_vertex_buffer_size.into(),
            VERTEX | COPY_DST,
        );
        changing_vertex_buffer_descriptor.label("changing storage for objects");
        let changing_vertex_buffer = gpu_device.create_buffer(&changing_vertex_buffer_descriptor);

        let k_color_offset = 0u32;
        let k_offset_offset = 1u32;
        
        let static_vertex_values_u8 = Uint8Array::new_with_length(static_vertex_buffer_size);
        let static_vertex_values_f32 = Float32Array::new(&static_vertex_values_u8.buffer());

        for i in 0..k_num_objects 
        {
            // let static_offset = i * (static_unit_size / 4);
            let static_offset_u8 = i * static_unit_size;
            let static_offset_f32 = static_offset_u8 / 4;

            // These are only set once so set them now
            let color = [
                (rand(None, None) * 255.0) as u8, 
                (rand(None, None) * 255.0) as u8, 
                (rand(None, None) * 255.0) as u8,
                255,
            ];
            let color_array = Uint8Array::new_with_length(color.len() as u32);
            color_array.copy_from(&color);
            static_vertex_values_u8.set(&color_array, static_offset_u8 + k_color_offset);    // set the color

            let offset = [
                rand(Some(-0.9), Some(0.9)), 
                rand(Some(-0.9), Some(0.9)),
            ];
            let offset_array = Float32Array::new_with_length(offset.len() as u32);
            offset_array.copy_from(&offset);
            static_vertex_values_f32.set(&offset_array, static_offset_f32 + k_offset_offset);  // set the offset

            object_infos.push(rand(Some(0.2), Some(0.5)));
        }
        gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &static_vertex_buffer, 0, &static_vertex_values_f32,
        );

        // a typed array we can use to update the changingStorageBuffer
        let changing_vertex_values = Float32Array::new_with_length(changing_vertex_buffer_size / 4);

        // setup a storage buffer with vertex data
        let (vertex_data, index_data, num_indexes) = 
            create_circle_vertices(Some(0.5), Some(0.25));

        let mut vertex_buffer_descriptor = GpuBufferDescriptor::new(
            vertex_data.byte_length().into(),
            VERTEX | COPY_DST,
        );
        vertex_buffer_descriptor.label("vertex buffer vertices");
        let vertex_buffer = gpu_device.create_buffer(&vertex_buffer_descriptor);
        gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &vertex_buffer, 0, &vertex_data,
        );

        let mut index_buffer_descriptor = GpuBufferDescriptor::new(
            index_data.byte_length().into(),
            INDEX | COPY_DST,
        );
        let index_buffer = gpu_device.create_buffer(&index_buffer_descriptor);
        index_buffer_descriptor.label("index buffer");
        gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &index_buffer, 0, &index_data,
        );

        Scene 
        {
            gpu_device, context, vertex_buffer, static_vertex_buffer, changing_vertex_buffer, index_buffer,
            render_pipeline, object_infos, changing_unit_size, changing_vertex_values, num_indexes, k_num_objects,
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
        render_pass_encoder.set_vertex_buffer(0, Some(&self.vertex_buffer));
        render_pass_encoder.set_vertex_buffer(1, Some(&self.static_vertex_buffer));
        render_pass_encoder.set_vertex_buffer(2, Some(&self.changing_vertex_buffer));
        render_pass_encoder.set_index_buffer(&self.index_buffer, GpuIndexFormat::Uint32);

        let canvas = self.context.canvas().dyn_into::<HtmlCanvasElement>().unwrap();
        let aspect = canvas.width() / canvas.height();
        let k_scale_offset = 0u32;

        for (ndx, scale) in self.object_infos.iter().enumerate()
        {
            let offset = ndx as u32 * (self.changing_unit_size / 4);

            let scale_vec = [scale / aspect as f32, *scale];
            let scale_array = Float32Array::new_with_length(scale_vec.len() as u32);
            scale_array.copy_from(&scale_vec);
            self.changing_vertex_values.set(&scale_array, offset + k_scale_offset);   // set the scale
        }
        self.gpu_device.queue().write_buffer_with_u32_and_buffer_source(
            &self.changing_vertex_buffer, 0, &self.changing_vertex_values,
        );

        render_pass_encoder.draw_indexed_with_instance_count(self.num_indexes, self.k_num_objects);

        render_pass_encoder.end();

        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<js_sys::Array>());
    }
}
