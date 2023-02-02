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

    var col = vec4<f32>(material.color1.xyz, 0.5);

 let thickness = 0.005;
     col = col * (1.0 -  smoothstep(0.0, thickness, uv.x))
         + col * (1.0 - smoothstep(0.0, thickness, 1.0 - uv.x))
         + col * (1.0 -  smoothstep(0.0, thickness, uv.y))
         + col * (1.0 - smoothstep(0.0, thickness, 1.0 - uv.y))

         + col * 0.3
         ;
;
    return col;
}