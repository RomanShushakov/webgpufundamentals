struct VertexShaderOutput 
{
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
};

@vertex 
fn vertex_main(@builtin(vertex_index) vertex_index : u32) -> VertexShaderOutput
{
    var pos = array(
        vec2f(1.0, 0.0),  // right, center
        vec2f(0.0, 0.0),  // center
        vec2f(1.0, 1.0),  // right, top
        vec2f(0.0, 1.0),  // center, top
    );

    var vs_output: VertexShaderOutput;
    let xy = pos[vertex_index];
    vs_output.position = vec4f(xy, 0.0, 1.0);
    vs_output.texcoord = xy;
    return vs_output;
}

@group(0) @binding(0) var our_sampler: sampler;
@group(0) @binding(1) var our_texture: texture_2d<f32>;

@fragment fn fragment_main(fs_input: VertexShaderOutput) -> @location(0) vec4f 
{
    var texture = textureSample(our_texture, our_sampler, fs_input.texcoord);
    texture.a = 0.5;
    return texture;
}


@vertex 
fn vertex_main_2(@builtin(vertex_index) vertex_index : u32) -> VertexShaderOutput
{
    var pos = array(
        vec2f(-1.0, 0.0),  // left, center
        vec2f(0.0, 0.0),  // center
        vec2f(-1.0, 1.0),  // left, top
        vec2f(0.0, 1.0),  // center, top
    );

    var vs_output: VertexShaderOutput;
    let xy = pos[vertex_index];
    vs_output.position = vec4f(xy, 0.0, 1.0);
    vs_output.texcoord = xy;
    return vs_output;
}


@fragment 
fn fragment_main_2(fs_input: VertexShaderOutput) -> @location(0) vec4f 
{
  return textureSample(our_texture, our_sampler, fs_input.texcoord);
}
