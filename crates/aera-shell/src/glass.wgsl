// AERA Glass: linear-light wallpaper, two filtering passes and optical material.
struct Params {
    pointer: vec2<f32>,
    radius: f32,
    dark: f32,
    aspect: f32,
    quality: f32,
    pad: vec2<f32>,
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var filtered: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;
@group(0) @binding(3) var<uniform> p: Params;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex fn vertex(@builtin(vertex_index) i: u32) -> VertexOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return VertexOut(vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0), uv);
}

// Procedural scene used in this milestone, not an arbitrary window capture.
@fragment fn scene(v: VertexOut) -> @location(0) vec4<f32> {
    let q = v.uv;
    let x = q.x * p.aspect;
    let wave = sin(x * 5.0 + q.y * 7.0);
    let bands = smoothstep(-0.15, 0.15, sin(length(vec2<f32>(x - p.aspect * 0.78, q.y - 1.0)) * 34.0));
    let color = mix(vec3<f32>(0.045, 0.10, 0.16), vec3<f32>(0.24, 0.48, 0.46), wave * 0.5 + 0.5);
    return vec4<f32>(color + bands * 0.08, 1.0);
}

// Kawase-style weighted center and four diagonal taps.
@fragment fn down(v: VertexOut) -> @location(0) vec4<f32> {
    let step = vec2<f32>(1.0) / vec2<f32>(textureDimensions(source));
    let d = step * (1.0 + p.quality * 2.0);
    var c = textureSample(source, linear_sampler, v.uv) * 4.0;
    c += textureSample(source, linear_sampler, v.uv + d);
    c += textureSample(source, linear_sampler, v.uv - d);
    c += textureSample(source, linear_sampler, v.uv + vec2<f32>(d.x, -d.y));
    c += textureSample(source, linear_sampler, v.uv + vec2<f32>(-d.x, d.y));
    return c / 8.0;
}

// Eight-tap reconstruction from half resolution.
@fragment fn up(v: VertexOut) -> @location(0) vec4<f32> {
    let d = vec2<f32>(1.0) / vec2<f32>(textureDimensions(source)) * (1.0 + p.quality);
    var c = textureSample(source, linear_sampler, v.uv + vec2<f32>(-2.0*d.x, 0.0));
    c += textureSample(source, linear_sampler, v.uv + vec2<f32>(2.0*d.x, 0.0));
    c += textureSample(source, linear_sampler, v.uv + vec2<f32>(0.0, -2.0*d.y));
    c += textureSample(source, linear_sampler, v.uv + vec2<f32>(0.0, 2.0*d.y));
    c += 2.0 * textureSample(source, linear_sampler, v.uv + d);
    c += 2.0 * textureSample(source, linear_sampler, v.uv - d);
    c += 2.0 * textureSample(source, linear_sampler, v.uv + vec2<f32>(d.x, -d.y));
    c += 2.0 * textureSample(source, linear_sampler, v.uv + vec2<f32>(-d.x, d.y));
    return c / 12.0;
}

// Superellipse implicit field; gradient supplies an optical normal.
// The panel occupies a central rectangle in the preview scene.
fn field(uv: vec2<f32>) -> f32 {
    let q = abs((uv - vec2<f32>(0.5)) / vec2<f32>(p.radius, 0.32));
    return pow(q.x, 5.0) + pow(q.y, 5.0) - 1.0;
}
@fragment fn glass(v: VertexOut) -> @location(0) vec4<f32> {
    let raw = textureSample(source, linear_sampler, v.uv).rgb;
    let f = field(v.uv);
    let grad = vec2<f32>(dpdx(f), dpdy(f));
    let pixel_distance = f / max(length(grad), 0.0001);
    let coverage = 1.0 - smoothstep(-0.5, 0.5, pixel_distance);
    let normal = grad / max(length(grad), 0.0001);
    let edge = exp(-abs(pixel_distance) / 9.0);
    let uv = clamp(v.uv - normal * edge * vec2<f32>(0.015 / p.aspect, 0.015), vec2<f32>(0.0), vec2<f32>(1.0));
    let blurred = textureSample(filtered, linear_sampler, uv).rgb;
    let mean = dot(textureSampleLevel(filtered, linear_sampler, vec2<f32>(0.5), 0.0).rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let tint = mix(vec3<f32>(0.80), vec3<f32>(0.035, 0.05, 0.065), p.dark);
    let glass_color = mix(blurred, tint, clamp(0.62 + (mean - 0.5) * 0.15, 0.55, 0.72));
    let direction = p.pointer - v.uv;
    let light = direction / max(length(direction), 0.0001);
    let specular = pow(max(dot(normal, light), 0.0), 4.0);
    let rim = exp(-abs(pixel_distance + 1.0) / 1.4) * (0.08 + 0.34 * specular);
    return vec4<f32>(mix(raw, glass_color + vec3<f32>(rim), coverage), 1.0);
}
