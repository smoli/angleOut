fn hash3( p: vec2<f32> ) -> vec3<f32>
{
    let q: vec3<f32> = vec3( dot(p,vec2(127.1,311.7)),
                   dot(p,vec2(269.5,183.3)),
                   dot(p,vec2(419.2,371.9)) );
    return fract(sin(q)*43758.5453);
}

fn voronoise( p: vec2<f32>, u: f32, v: f32 ) -> f32
{
    let k: f32 = 1.0+63.0*pow(1.0-v,6.0);

    let i: vec2<f32>= floor(p);
    let f: vec2<f32>= fract(p);

    var a: vec2<f32>= vec2(0.0,0.0);
    for (var y = -2; y<=2; y++) {
    for (var x = -2; x<=2; x++)
    {
       let g: vec2<f32> = vec2<f32>( f32(x), f32(y) );
        let o: vec3<f32> = hash3( i + g ) * vec3(u,u,1.0);
        let d: vec2<f32> = g - f + o.xy;
        let w: f32 = pow( 1.0-smoothstep(0.0,1.414,length(d)), k );
        a += vec2(o.z*w,w);
    }
    }

    return a.x/a.y;
}


struct CustomMaterial {
    color1: vec4<f32>,
    color2: vec4<f32>,
    time: f32
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
/*@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;*/


fn f5(x: f32) -> f32 {
    return floor(x * 2.0) * 0.25;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {

/*    let tempo: f32 = 0.5;
    let x = material.time * tempo;
    let p: f32 =  sin(x) * 0.5 + 0.5;
    return material.color1 * vec4<f32>(1.0, 1.0, 1.0, p);*/
//
//    let col = vec3<f32>(1.0, 0.5, 0.5);
//
//    return vec4<f32>(col * uv.x, 1.0);

    var col = vec3<f32>(material.color1.xyz);
/*
    col = col * 0.01 / uv.x
         + col * 0.01 / (1.0 - uv.x)
         + col * 0.01 / uv.y
         + col * 0.01 / (1.0 - uv.y);*/



    let pix = floor(uv * 5.0);

 let thickness = 0.05;
     col = col * smoothstep(1.0 - thickness, 1.0, uv.x)
         + col * smoothstep(1.0 - thickness, 1.0, uv.y)
         + col * smoothstep(1.0 - thickness, 1.0, 1.0 - uv.x)
         + col * smoothstep(1.0 - thickness, 1.0, 1.0 - uv.y)

         + mix(col * 0.8, col * 1.0, sin(material.time + pix.x - pix.y) + cos(material.time + pix.x * pix.y));


    return vec4<f32>(col, 1.0);
    //return vec4<f32>(uv, 0.5 + 0.5 * sin(material.time), 1.0) * uv.y;
}