#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::{view, globals},
    pbr_types::{PbrInput, pbr_input_new, STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, prepare_world_normal, calculate_view},
}

// Keep in step with FORCE_FIELD_HIT_SLOTS / FORCE_FIELD_TIME_WRAP in
// src/materials/force_field.rs. `globals.time` is Bevy's wrapped clock, so hit
// times are recorded with `Time::elapsed_secs_wrapped()` and aged the same way.
const HIT_SLOTS: u32 = 8u;
const TIME_WRAP: f32 = 3600.0;

struct ForceFieldMaterial {
    sheet_color: vec4<f32>,
    flare_color: vec4<f32>,
    panel_size: vec2<f32>,
    ripple_speed: f32,
    ripple_width: f32,
    ripple_decay: f32,
    flare_intensity: f32,
    hex_tile_size: f32,
    _padding: f32,
    hits: array<vec4<f32>, 8>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: ForceFieldMaterial;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var color_sampler: sampler;

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

    return mix(a, b, u.x) +
            (c - a) * u.y * (1.0 - u.x) +
            (d - b) * u.x * u.y;
}

/// How long ago a hit was recorded, on the hourly-wrapping clock.
fn hit_age(start: f32) -> f32 {
    let age = globals.time - start;

    if age < 0.0 {
        return age + TIME_WRAP;
    }

    return age;
}

/// Summed strength of every live ripple at panel position `p` (world units).
/// Each slot contributes a gaussian ring expanding at `ripple_speed`, so two
/// impacts inside one lifetime read as two overlapping wavefronts.
fn ripple_strength(p: vec2<f32>) -> f32 {
    var wave = 0.0;

    for (var i = 0u; i < HIT_SLOTS; i++) {
        let hit = material.hits[i];

        if hit.w < 0.5 {
            continue;
        }

        let age = hit_age(hit.z);

        if age >= material.ripple_decay {
            continue;
        }

        let d = distance(p, hit.xy * material.panel_size);
        let front = (d - age * material.ripple_speed) / material.ripple_width;
        // Ripples simply die at the panel edges rather than reflecting off them.
        let decay = 1.0 - age / material.ripple_decay;

        wave += exp(-front * front) * decay * decay;
    }

    return wave;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    // Panel-local position in world units, so ripples stay round on a panel
    // that is ten times wider than it is tall.
    let p = in.uv * material.panel_size;

    // At rest: a smooth energy sheet, slowly drifting, with no lattice on it.
    let drift = vec2<f32>(globals.time * 0.02, globals.time * 0.05);
    let shimmer = noise(in.uv * vec2<f32>(6.0, 2.0) + drift) * 0.5
        + noise(in.uv * vec2<f32>(13.0, 4.0) - drift * 1.7) * 0.25;
    let sheet = 0.12 + 0.10 * shimmer;

    // The sheet feathers out along the top and bottom edges; the left and right
    // ends stay hot so the arena bounds still read during play.
    let fade = smoothstep(0.0, 1.0, in.uv.y * 10.0)
        * smoothstep(0.0, 1.0, (1.0 - in.uv.y) * 10.0);
    let rim = clamp(
        smoothstep(1.0, 0.0, in.uv.x * 100.0) + smoothstep(1.0, 0.0, (1.0 - in.uv.x) * 100.0),
        0.0,
        1.0,
    );

    // The lattice only exists where a ripple is currently passing over it.
    let wave = ripple_strength(p);
    let hex = textureSample(color_texture, color_sampler, p / material.hex_tile_size).x;
    let flare = hex * wave * material.flare_intensity;

    var emissive = material.sheet_color.xyz * (sheet + rim * 1.5);
    emissive += material.flare_color.xyz * (flare + saturate(wave) * 0.35);

    let alpha = clamp(
        (sheet + saturate(wave) * 0.7 + saturate(flare) * 0.5) * fade + rim,
        0.0,
        1.0,
    );

    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = vec4<f32>(1.0, 1.0, 1.0, alpha);

    pbr_input.material.reflectance = vec3<f32>(1.0);
    pbr_input.material.alpha_cutoff = 0.0;
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT;
    pbr_input.material.emissive = vec4<f32>(emissive, alpha);
    pbr_input.material.metallic = 0.1;
    pbr_input.material.perceptual_roughness = 1.0;

    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = in.world_normal;

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;

    pbr_input.N = prepare_world_normal(in.world_normal, false, is_front);
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    let output_color = apply_pbr_lighting(pbr_input);

    return main_pass_post_lighting_processing(pbr_input, output_color);
}
