struct Vertex {
    @location(0) position: vec2f,
    @location(1) color: vec4f,
    @location(2) offset: vec2f,
    @location(3) scale: vec2f,
};

struct VSOutput {
  @builtin(position) position: vec4f,
  @location(0) color: vec4f,
}


@vertex 
fn vertex_main(
    vert: Vertex,
) 
    -> VSOutput
{
    var vs_out: VSOutput;
    vs_out.position = vec4f(vert.position * vert.scale + vert.offset, 0.0, 1.0);
    vs_out.color = vert.color;
    return vs_out;
}


@fragment 
fn fragment_main(vs_out: VSOutput) -> @location(0) vec4f
{
    return vs_out.color;
}
