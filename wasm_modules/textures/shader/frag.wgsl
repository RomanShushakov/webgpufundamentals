struct VertexShaderOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
};

@group(0) @binding(0) var our_sampler: sampler;
@group(0) @binding(1) var our_texture: texture_2d<f32>;

@fragment fn fragment_main(fs_input: VertexShaderOutput) -> @location(0) vec4f 
{
  return textureSample(our_texture, our_sampler, fs_input.texcoord);
}
