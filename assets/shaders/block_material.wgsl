#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

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
    damage: f32,
    time: f32,
    top_bottom_split: i32
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var color_texture: texture_2d<f32>;
@group(1) @binding(2)
var color_sampler: sampler;


fn f5(x: f32) -> f32 {
    return floor(x * 2.0) * 0.25;
}


struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(
    in: FragmentInput
) -> @location(0) vec4<f32> {

    let damage = material.damage;
    var color: vec4<f32>;

    if (material.top_bottom_split == 1 && in.uv.y > 0.5) {
        color = material.color2;
    } else {
        color = material.color1;
    }

    if (damage != 0.0) {
        var s:vec4<f32> = vec4<f32>(0.0);
        var uv = vec2<f32>(in.uv);

        s = s +textureSample(color_texture, color_sampler,uv / 2.0);

        if (damage > 1.0) {
            uv = vec2<f32>(in.uv.y, in.uv.x);
            s = s * 0.5 + textureSample(color_texture, color_sampler, uv / 2.0) * 0.5;
        }


        color = vec4<f32>(color.xyz * step(0.1, s.x), 1.0);
    }


        var pbr_input: PbrInput;

        pbr_input.material.base_color = vec4<f32>(color);

        pbr_input.material.reflectance = 0.1;
        pbr_input.material.alpha_cutoff = 0.0;
        pbr_input.material.flags = 4u;
        pbr_input.material.emissive = vec4<f32>(0.0,0.0,0.0,1.0);
        pbr_input.material.metallic = 0.1;
        pbr_input.material.perceptual_roughness = 1.0;

        pbr_input.occlusion = 1.0;
        pbr_input.frag_coord = in.frag_coord;
        pbr_input.world_position = in.world_position;
        pbr_input.world_normal = in.world_normal;

        pbr_input.is_orthographic = view.projection[3].w == 1.0;

        pbr_input.N = prepare_world_normal(in.world_normal, false, in.is_front);
        pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic)
        ;

        let output_color = pbr(pbr_input);

        return tone_mapping(pbr(pbr_input));
}