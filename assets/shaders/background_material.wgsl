struct CustomMaterial {
    color1: vec4<f32>,
    color2: vec4<f32>,
    time: f32
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;



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
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    return  vec4<f32>(voronoi(uv), 1.0);
}



/*
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
}*/
