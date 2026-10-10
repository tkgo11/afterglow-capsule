// SPIKE ONLY. Synthetic public scene; no content, assets or release state.
struct Params { size_time_mode: vec4<f32>, pointer: vec4<f32> }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var scene: texture_2d<f32>;
@group(0) @binding(2) var scene_sampler: sampler;

struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    let p = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var output: VertexOutput;
    output.position = vec4(p[index], 0.0, 1.0);
    output.uv = p[index] * vec2(0.5,-0.5) + vec2(0.5);
    return output;
}

@fragment fn atmosphere(input: VertexOutput) -> @location(0) vec4<f32> {
    let mode = params.size_time_mode.w;
    let t = select(params.size_time_mode.z, 0.0, mode >= 2.0);
    let wave = sin(input.uv.x * 8.0 + t * 0.2) * 0.06;
    let aurora = exp(-abs(input.uv.y - 0.3 - wave) * 10.0);
    let color = vec3(0.015,0.022,0.04) + vec3(0.03,0.12,0.15) * aurora;
    return vec4(color, 1.0);
}

fn sampled(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(scene, scene_sampler, uv, 0.0).rgb;
}

@fragment fn glass(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let mode = params.size_time_mode.w;
    var color = sampled(uv);
    // Separate controls from a mostly opaque synthetic reading surface.
    let control = uv.x > 0.07 && uv.x < 0.3 && uv.y > 0.18 && uv.y < 0.8;
    if control {
        let edge_distance = min(min(uv.x-0.07,0.3-uv.x),min(uv.y-0.18,0.8-uv.y));
        let inner_edge = 1.0 - smoothstep(0.001,0.008,edge_distance);
        if mode >= 2.0 {
            color = mix(vec3(0.065,0.09,0.13),vec3(0.5,0.65,0.7),inner_edge);
        } else {
            // Edge-biased refraction, diffusion, luminance adaptation, tint,
            // inner edge, specular, restrained dispersion, then foreground.
            let refraction = (uv-vec2(0.185,0.49)) * inner_edge * 0.025;
            let refracted = uv + refraction;
            let radius = select(0.004,0.001,mode >= 1.0);
            var diffuse = sampled(refracted) * 0.4;
            diffuse += sampled(refracted + vec2(radius,0.0)) * 0.15;
            diffuse += sampled(refracted - vec2(radius,0.0)) * 0.15;
            diffuse += sampled(refracted + vec2(0.0,radius)) * 0.15;
            diffuse += sampled(refracted - vec2(0.0,radius)) * 0.15;
            let luminance = dot(diffuse,vec3(0.2126,0.7152,0.0722));
            let adapted = diffuse * (0.55 / max(luminance,0.3));
            color = mix(adapted,vec3(0.08,0.13,0.18),0.35);
            color += inner_edge * vec3(0.17,0.23,0.25);
            color += pow(max(0.0,1.0-uv.y),8.0) * inner_edge * 0.25;
            if mode < 1.0 {
                color.r += (sampled(refracted+vec2(0.001,0.0)).r-diffuse.r)*0.15;
                color.b += (sampled(refracted-vec2(0.001,0.0)).b-diffuse.b)*0.15;
            }
        }
    }
    if uv.x > 0.36 && uv.x < 0.93 && uv.y > 0.18 && uv.y < 0.8 {
        color = vec3(0.09,0.11,0.14);
        // Synthetic stable foreground bars exercise content/effect separation.
        if uv.x > 0.4 && uv.x < 0.88 && fract((uv.y-0.23)*16.0) < 0.08 && uv.y < 0.72 && uv.y > 0.23 {
            color = vec3(0.72,0.78,0.84);
        }
    }
    // Pointer feedback is opaque and survives every effect fallback.
    if distance(uv,params.pointer.xy) < 0.009 { color = vec3(0.9,0.95,1.0); }
    return vec4(color,1.0);
}
