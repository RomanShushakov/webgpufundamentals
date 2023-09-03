struct OurStruct {
    color: vec4f,
    scale: vec2f,
    offset: vec2f,
};
 
@group(0) @binding(0) var<uniform> our_struct: OurStruct;


@vertex 
fn vertex_main(@builtin(vertex_index) vertex_index : u32) -> @builtin(position) vec4f
{
    var pos = array(
        vec2(0.0, 0.5),
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5),
    );

    return vec4f(pos[vertex_index] * our_struct.scale + our_struct.offset, 0.0, 1.0);
}


@fragment 
fn fragment_main() -> @location(0) vec4f
{
    return our_struct.color;
}
