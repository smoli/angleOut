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

    var col = vec3<f32>(1.0, 0.0, 1.0);

 let thickness = 0.005;
     col = col * smoothstep(1.0 - thickness, 1.0, uv.x)
         + col * smoothstep(1.0 - thickness, 1.0, 1.0 - uv.x)
        ;

    return vec4<f32>(col, 1.0);
}