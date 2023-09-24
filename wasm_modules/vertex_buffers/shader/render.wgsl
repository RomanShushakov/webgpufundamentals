struct VSOutput {
  @builtin(position) position: vec4f,
  @location(0) color: vec4f,
}


@vertex 
fn vertex_main(
    @location(0) position: vec2f,
    @location(1) color: vec4f,
    @location(2) offset: vec2f,
    @location(3) scale: vec2f,
    @location(4) per_vertex_color: vec4f,
) 
    -> VSOutput
{
    var vs_out: VSOutput;
    vs_out.position = vec4f(position * scale + offset, 0.0, 1.0);
    vs_out.color = color * per_vertex_color;
    return vs_out;
}


@fragment 
fn fragment_main(vs_out: VSOutput) -> @location(0) vec4f
{
    return vs_out.color;
}
