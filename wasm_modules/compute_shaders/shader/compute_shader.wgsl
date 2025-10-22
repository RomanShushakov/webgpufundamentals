@group(0) @binding(0) var<storage, read_write> workgroup_result: array<vec3u>;
@group(0) @binding(1) var<storage, read_write> local_result: array<vec3u>;
@group(0) @binding(2) var<storage, read_write> global_result: array<vec3u>;
 
@compute @workgroup_size(${workgroup_size}) fn compute_main(
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
    @builtin(local_invocation_id) local_invocation_id : vec3<u32>,
    @builtin(global_invocation_id) global_invocation_id : vec3<u32>,
    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
)
{
    // workgroup_index is similar to local_invocation_index except for
    // workgroups, not threads inside a workgroup.
    // It is not a builtin so we compute it ourselves.
    let workgroup_index =
        workgroup_id.x +
        workgroup_id.y * num_workgroups.x +
        workgroup_id.z * num_workgroups.x * num_workgroups.y;

    // global_invocation_index is like local_invocation_index
    // except linear across all invocations across all dispatched
    // workgroups. It is not a builtin so we compute it ourselves.
    let global_invocation_index =
        workgroup_index * ${num_threads_per_workgroup} +
        local_invocation_index;
 
    // now we can write each of these builtins to our buffers.
    workgroup_result[global_invocation_index] = workgroup_id;
    local_result[global_invocation_index] = local_invocation_id;
    global_result[global_invocation_index] = global_invocation_id;
}
