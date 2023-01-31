struct CustomMaterial {
    color1: vec4<f32>,
    color2: vec4<f32>,
    time: f32
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {

    var col = vec3<f32>(0.0, 0.05, 0.5);
    var q = uv - vec2<f32>(0.5, 0.5);

    let r = 0.1;
    col = col * smoothstep(r, r + 0.01, length( q ) + 10.0 * atan2(uv.y, uv.x));

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);

    //return vec4<f32>(uv, 0.5 + 0.5 * sin(material.time), 1.0) * uv.y;
}