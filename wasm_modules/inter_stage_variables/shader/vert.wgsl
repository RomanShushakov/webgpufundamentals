struct VertexShaderOutput {
    @builtin(position) position: vec4f,
};

@vertex 
fn vertex_main(@builtin(vertex_index) vertex_index : u32) -> VertexShaderOutput
{
    var pos = array(
        vec2(0.0, 0.5),
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5),
    );

    var vs_output: VertexShaderOutput;
    vs_output.position = vec4(pos[vertex_index], 0.0, 1.0);

    return vs_output;
}
