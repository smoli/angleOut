#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

struct CustomMaterial {
    color1: vec4<f32>,
    hit_position: vec3<f32>,
    hit_time: f32,
    time: f32
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@group(1) @binding(1)
var color_texture: texture_2d<f32>;
@group(1) @binding(2)
var color_sampler: sampler;

fn random1(p: f32) -> f32 {
    return fract(
        sin(dot(vec2<f32>(p), vec2<f32>(12.9898, 78.233)))
        * 43758.5453123
    );
}
fn random(p: vec2<f32>) -> f32 {
    return fract(
        sin(dot(p, vec2<f32>(12.9898, 78.233)))
        * 43758.5453123
    );
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let a = random(i);
    let b = random(i + vec2<f32>(1.0, 0.0));
    let c = random(i + vec2<f32>(0.0, 1.0));
    let d = random(i + vec2<f32>(1.0, 1.0));

    let u = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), f);
//    let u = f * f * (3.0 - 2.0 * f);


    return mix(a, b, u.x) +
            (c - a) * u.y * (1.0 - u.x) +
            (d - b) * u.x * u.y;
}

fn voronoi(p: vec2<f32>) -> vec3<f32> {
    var color = vec3<f32>(0.0);

    var points: array<vec2<f32>, 4>;

    points[0] = vec2<f32>(0.83, 0.75);
    points[1] = vec2<f32>(0.60, 0.07);
    points[2] = vec2<f32>(0.28, 0.64);
    points[3] = vec2<f32>(0.31, 0.26);

    var m_dist = 1.0;
    let ft = fract(material.time);

    let tr = vec2<f32>(
        0.0,
        (0.2 * material.time),
    );

    for (var i = 0; i < 4; i++) {
        m_dist = min(m_dist, distance(p,

        points[i]

        + noise((p + tr) * 5.0)));
    }

    color += m_dist;
    //color -= step(0.7, abs(sin(50.0 * m_dist))) * 0.2;

    return color;
}

@fragment
fn fragment(
    in: FragmentInput
) -> @location(0) vec4<f32> {

    let ar = vec2<f32>(1.0, 0.1);

    let t = textureSample(color_texture, color_sampler, (in.uv * ar * 5.5) % 1.0);

    let color = vec4<f32>(material.color1.xyz, (t.x) * 0.5 + 0.2 * smoothstep(0.0, 1.0, noise(material.time + in.uv * ar * 7.0)));


    var pbr_input: PbrInput;

    pbr_input.material.base_color = vec4<f32>(1.0, 1.0, 1.0, color.a);

    pbr_input.material.reflectance = 0.1;
    pbr_input.material.alpha_cutoff = 0.0;
    pbr_input.material.flags = 2u | 4u;
    pbr_input.material.emissive = color;
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