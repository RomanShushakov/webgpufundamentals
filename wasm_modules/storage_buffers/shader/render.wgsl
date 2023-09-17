struct OurStruct {
    color: vec4f,
    offset: vec2f,
};

struct OtherStruct {
    scale: vec2f,
};

struct VSOutput {
  @builtin(position) position: vec4f,
  @location(0) color: vec4f,
}
 
@group(0) @binding(0) var<storage, read> our_structs: array<OurStruct>;
@group(0) @binding(1) var<storage, read> other_structs: array<OtherStruct>;


@vertex 
fn vertex_main(
    @builtin(vertex_index) vertex_index : u32,
    @builtin(instance_index) instance_index: u32,
) 
    -> VSOutput
{
    var pos = array(
        vec2(0.0, 0.5),
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5),
    );

    var our_struct = our_structs[instance_index];
    var other_struct = other_structs[instance_index];

    var vs_out: VSOutput;
    vs_out.position = vec4f(
        pos[vertex_index] * other_struct.scale + our_struct.offset, 0.0, 1.0);
    vs_out.color = our_struct.color;
    return vs_out;
}


@fragment 
fn fragment_main(vs_out: VSOutput) -> @location(0) vec4f
{
    return vs_out.color;
}
