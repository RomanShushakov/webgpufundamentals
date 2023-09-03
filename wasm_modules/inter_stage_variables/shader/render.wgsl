struct OurVertexShaderOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
};


@vertex 
fn vertex_main(@builtin(vertex_index) vertex_index : u32) -> OurVertexShaderOutput
{
    var pos = array(
        vec2(0.0, 0.5),
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5),
    );

    var color = array(
        vec4(1.0, 0.0, 0.0, 1.0),
        vec4(0.0, 1.0, 0.0, 1.0),
        vec4(0.0, 0.0, 1.0, 1.0),
    );

    var vs_output: OurVertexShaderOutput;
    vs_output.position = vec4(pos[vertex_index], 0.0, 1.0);
    vs_output.color = color[vertex_index];
    return vs_output;
}
 

@fragment 
fn fragment_main(fs_input: OurVertexShaderOutput) -> @location(0) vec4f 
{
    return fs_input.color;
}
