@fragment 
fn fragment_main(@builtin(position) pixel_position: vec4f) -> @location(0) vec4f
{
    let red = vec4(1.0, 0.0, 0.0, 1.0);
    let cyan = vec4(0.0, 1.0, 1.0, 1.0);

    let grid = vec2u(pixel_position.xy) / 8u;
    let checker = (grid.x + grid.y) % 2u == 1u;

    return select(red, cyan, checker);
}
