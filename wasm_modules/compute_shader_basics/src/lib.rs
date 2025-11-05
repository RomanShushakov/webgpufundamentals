use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::
{
    GpuDevice, GpuShaderModuleDescriptor, GpuComputePipelineDescriptor, GpuProgrammableStage, GpuBufferDescriptor,
    GpuBindGroupDescriptor, GpuBindGroupEntry, GpuBufferBinding, GpuComputePassDescriptor, GpuBufferBindingLayout,
    GpuBindGroupLayoutEntry, GpuBindGroupLayoutDescriptor, GpuPipelineLayoutDescriptor, GpuBufferBindingType,
};
use web_sys::gpu_buffer_usage::{COPY_SRC, COPY_DST, STORAGE, MAP_READ};
use web_sys::gpu_map_mode::READ;
use web_sys::gpu_shader_stage::COMPUTE;
use js_sys::{Uint32Array, Object, Reflect, Array};
use futures::future::join3;


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
}


#[wasm_bindgen]
impl Scene
{
    pub fn create(
        gpu_device: GpuDevice,
    ) 
        -> Self
    {
        Scene 
        {
            gpu_device,
        }
    }


    pub async fn compute(&self) -> Result<JsValue, JsValue>
    {
        let dispatch_count = [4u32, 3, 2];
        let workgroup_size = [2u32, 3, 4];

        // multiply all elements of an array
        let array_prod = |arr: &[u32]| arr.iter().fold(1, |acc, x| acc * x);

        let num_threads_per_workgroup = array_prod(&workgroup_size);

        let compute_shader_code = include_str!("../shader/compute_shader.wgsl")
            .replace("${workgroup_size}", &format!("{:?}", workgroup_size).replace("[", "").replace("]", ""))
            .replace("${num_threads_per_workgroup}", &format!("{:?}", num_threads_per_workgroup));

        let num_workgroups = array_prod(&dispatch_count);
        let num_results = num_workgroups * num_threads_per_workgroup;
        let size = num_results * 4 * 4;  // vec3f * u32


        // source buffers
        let src_buffer_usage = STORAGE | COPY_SRC;
        let workgroup_buffer_descriptor = GpuBufferDescriptor::new(
            size.into(),
            src_buffer_usage,
        );
        workgroup_buffer_descriptor.set_label("workgroup buffer");
        let workgroup_buffer = self.gpu_device.create_buffer(&workgroup_buffer_descriptor)?;

        let local_buffer_descriptor = workgroup_buffer_descriptor.clone();
        local_buffer_descriptor.set_label("local buffer");
        let local_buffer = self.gpu_device.create_buffer(&local_buffer_descriptor)?;

        let global_buffer_descriptor = workgroup_buffer_descriptor.clone();
        global_buffer_descriptor.set_label("global buffer");
        let global_buffer = self.gpu_device.create_buffer(&global_buffer_descriptor)?;


        // destination buffers
        let dst_buffer_usage = MAP_READ | COPY_DST;
        let workgroup_read_buffer_descriptor = GpuBufferDescriptor::new(
            size.into(),
            dst_buffer_usage,
        );
        workgroup_read_buffer_descriptor.set_label("workgroup read buffer");
        let workgroup_read_buffer = self.gpu_device.create_buffer(&workgroup_read_buffer_descriptor)?;

        let local_read_buffer_descriptor = workgroup_read_buffer_descriptor.clone();
        local_read_buffer_descriptor.set_label("local read buffer");
        let local_read_buffer = self.gpu_device.create_buffer(&local_read_buffer_descriptor)?;

        let global_read_buffer_descriptor = workgroup_read_buffer_descriptor.clone();
        global_read_buffer_descriptor.set_label("global read buffer");
        let global_read_buffer = self.gpu_device.create_buffer(&global_read_buffer_descriptor)?;


        let compute_shader_module_descriptor = GpuShaderModuleDescriptor::new(&compute_shader_code);
        compute_shader_module_descriptor.set_label("compute module");
        let compute_shader_module = self.gpu_device.create_shader_module(&compute_shader_module_descriptor);
        let compute_stage = GpuProgrammableStage::new(&compute_shader_module);


        // Define workgroup_result bind group layout
        let workgroup_result_binding_layout = GpuBufferBindingLayout::new();
        workgroup_result_binding_layout.set_type(GpuBufferBindingType::Storage);
        let bind_group_0_layout_entry_0 = GpuBindGroupLayoutEntry::new(
            0, COMPUTE,
        );
        bind_group_0_layout_entry_0.set_buffer(&workgroup_result_binding_layout);

        // Define local_result bind group layout
        let local_result_binding_layout = GpuBufferBindingLayout::new();
        local_result_binding_layout.set_type(GpuBufferBindingType::Storage);
        let bind_group_0_layout_entry_1 = GpuBindGroupLayoutEntry::new(
            1, COMPUTE,
        );
        bind_group_0_layout_entry_1.set_buffer(&local_result_binding_layout);

        // Define global_result bind group layout
        let global_result_binding_layout = GpuBufferBindingLayout::new();
        global_result_binding_layout.set_type(GpuBufferBindingType::Storage);
        let bind_group_0_layout_entry_2 = GpuBindGroupLayoutEntry::new(
            2, COMPUTE,
        );
        bind_group_0_layout_entry_2.set_buffer(&global_result_binding_layout);

        let bind_group_0_layout_descriptor = GpuBindGroupLayoutDescriptor::new(
            &[
                &bind_group_0_layout_entry_0,
                &bind_group_0_layout_entry_1,
                &bind_group_0_layout_entry_2,
            ].iter().collect::<Array>(),
        );
        let bind_group_0_layout = self.gpu_device
            .create_bind_group_layout(&bind_group_0_layout_descriptor)?;

        let pipeline_layout_descriptor = GpuPipelineLayoutDescriptor::new(
            &[&bind_group_0_layout].iter().collect::<Array>(),
        );
        let compute_layout = self.gpu_device.create_pipeline_layout(
            &pipeline_layout_descriptor,
        );

        // let compute_layout = JsValue::from("auto");

        let compute_pipeline_descriptor = GpuComputePipelineDescriptor::new(
            &compute_layout,
            &compute_stage,
        );
        compute_pipeline_descriptor.set_label("compute pipeline");
        let compute_pipeline = self.gpu_device.create_compute_pipeline(&compute_pipeline_descriptor);

        let bind_group_0_entry_0_resource = GpuBufferBinding::new(&workgroup_buffer);
        let bind_group_0_entry_0 = GpuBindGroupEntry::new(0, &bind_group_0_entry_0_resource);
        let bind_group_0_entry_1_resource = GpuBufferBinding::new(&local_buffer);
        let bind_group_0_entry_1 = GpuBindGroupEntry::new(1, &bind_group_0_entry_1_resource);
        let bind_group_0_entry_2_resource = GpuBufferBinding::new(&global_buffer);
        let bind_group_0_entry_2 = GpuBindGroupEntry::new(2, &bind_group_0_entry_2_resource);
        let bind_group_0_entries = [
            bind_group_0_entry_0,
            bind_group_0_entry_1,
            bind_group_0_entry_2,
        ].iter().collect::<Array>();
        // let compute_bind_group_0_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group_0_descriptor = GpuBindGroupDescriptor::new(
            &bind_group_0_entries,
            // &compute_bind_group_0_layout,
            &bind_group_0_layout,
        );
        let bind_group_0 = self.gpu_device.create_bind_group(&bind_group_0_descriptor);


        let command_encoder = self.gpu_device.create_command_encoder();
        command_encoder.set_label("compute builtin encoder");
        let compute_pass_descriptor = GpuComputePassDescriptor::new();
        compute_pass_descriptor.set_label("compute builtin pass");
        let compute_pass = command_encoder.begin_compute_pass_with_descriptor(&compute_pass_descriptor);
        

        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, Some(&bind_group_0));
        compute_pass.dispatch_workgroups_with_workgroup_count_y_and_workgroup_count_z(
            dispatch_count[0], dispatch_count[1], dispatch_count[2],
        );
        compute_pass.end();


        command_encoder.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            &workgroup_buffer, 0, &workgroup_read_buffer, 0, size,
        )?;
        command_encoder.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            &local_buffer, 0, &local_read_buffer, 0, size,
        )?;
        command_encoder.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            &global_buffer, 0, &global_read_buffer, 0, size,
        )?;


        let command_buffer = command_encoder.finish();
        self.gpu_device.queue().submit(&[command_buffer].iter().collect::<Array>());

        let _ = join3(
            JsFuture::from(workgroup_read_buffer.map_async(READ)),
            JsFuture::from(local_read_buffer.map_async(READ)),
            JsFuture::from(global_read_buffer.map_async(READ)),
        ).await;

        let workgroup_read_result_buffer = workgroup_read_buffer.get_mapped_range()?;
        let workgroup_read_output = Uint32Array::new(&workgroup_read_result_buffer.slice(0));
        workgroup_read_buffer.unmap();

        let local_read_result_buffer = local_read_buffer.get_mapped_range()?;
        let local_read_output = Uint32Array::new(&local_read_result_buffer.slice(0));
        local_read_buffer.unmap();

        let global_read_result_buffer = global_read_buffer.get_mapped_range()?;
        let global_read_output = Uint32Array::new(&global_read_result_buffer.slice(0));
        global_read_buffer.unmap();

        let output_object = Object::new();
        Reflect::set(&output_object, &"num_results".into(), &num_results.into())?;
        Reflect::set(&output_object, &"num_threads_per_workgroup".into(), &num_threads_per_workgroup.into())?;
        Reflect::set(&output_object, &"workgroup_read_output".into(), &workgroup_read_output)?;
        Reflect::set(&output_object, &"local_read_output".into(), &local_read_output)?;
        Reflect::set(&output_object, &"global_read_output".into(), &global_read_output)?;

        Ok(output_object.into())
    }
}
