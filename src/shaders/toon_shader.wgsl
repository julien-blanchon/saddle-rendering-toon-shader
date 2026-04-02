#import bevy_pbr::{
    mesh_view_bindings as view_bindings,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    pbr_deferred_functions::deferred_output,
    prepass_io::{FragmentOutput, VertexOutput},
}
#else
#import bevy_pbr::forward_io::{FragmentOutput, VertexOutput}
#endif

struct ToonExtensionUniform {
    shadow_tint: vec4<f32>,
    specular_color: vec4<f32>,
    rim_color: vec4<f32>,
    diffuse_mode: u32,
    band_count: u32,
    rim_lit_only: u32,
    _padding_mode: u32,
    band_softness: f32,
    shadow_floor: f32,
    light_wrap: f32,
    _padding_a: f32,
    specular_threshold: f32,
    specular_softness: f32,
    specular_intensity: f32,
    specular_width: f32,
    rim_threshold: f32,
    rim_softness: f32,
    rim_intensity: f32,
    _padding_b: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> toon: ToonExtensionUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var ramp_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var ramp_sampler: sampler;

fn saturate_f32(value: f32) -> f32 {
    return clamp(value, 0.0, 1.0);
}

fn max_component(value: vec3<f32>) -> f32 {
    return max(max(value.r, value.g), value.b);
}

fn luminance(value: vec3<f32>) -> f32 {
    return dot(value, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn normalized_tint(value: vec3<f32>) -> vec3<f32> {
    let peak = max(max_component(value), 1e-4);
    return value / peak;
}

fn quantize_bands(value: f32, band_count: u32, softness: f32) -> f32 {
    let steps = max(f32(band_count - 1u), 1.0);
    let scaled = saturate_f32(value) * steps;
    let lower = floor(scaled);
    let upper = min(lower + 1.0, steps);
    let blend = smoothstep(
        0.5 - softness * 0.5,
        0.5 + softness * 0.5,
        fract(scaled),
    );
    return mix(lower / steps, upper / steps, blend);
}

fn band_visibility(value: f32) -> f32 {
    return max(
        toon.shadow_floor,
        quantize_bands(value + toon.light_wrap, toon.band_count, toon.band_softness),
    );
}

fn shadowed_base_color(base_color: vec3<f32>) -> vec3<f32> {
    return base_color * toon.shadow_tint.rgb;
}

fn diffuse_shading(base_color: vec3<f32>, diffuse_lit: vec3<f32>, value: f32) -> vec4<f32> {
    if toon.diffuse_mode == 1u {
        let ramp = textureSample(ramp_texture, ramp_sampler, vec2<f32>(saturate_f32(value), 0.5));
        let visibility = max(toon.shadow_floor, saturate_f32(luminance(ramp.rgb)));
        let ramp_tinted_lit = diffuse_lit * mix(vec3<f32>(1.0), ramp.rgb, 0.85);
        return vec4<f32>(
            mix(shadowed_base_color(base_color), ramp_tinted_lit, visibility),
            visibility,
        );
    }

    let visibility = band_visibility(value);
    return vec4<f32>(
        mix(shadowed_base_color(base_color), diffuse_lit, visibility),
        visibility,
    );
}

fn diffuse_probe_input(input: pbr_types::PbrInput) -> pbr_types::PbrInput {
    var probe = input;
    probe.material.emissive = vec4<f32>(0.0);
    probe.material.metallic = 0.0;
    probe.material.reflectance = 0.0;
    probe.material.specular_transmission = 0.0;
    probe.material.diffuse_transmission = 0.0;
#ifdef STANDARD_MATERIAL_CLEARCOAT
    probe.material.clearcoat = 0.0;
    probe.material.clearcoat_perceptual_roughness = 1.0;
#endif
    return probe;
}

fn full_probe_input(input: pbr_types::PbrInput) -> pbr_types::PbrInput {
    var probe = input;
    probe.material.emissive = vec4<f32>(0.0);
    return probe;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material,
        pbr_input.material.base_color,
    );

#ifdef PREPASS_PIPELINE
    return deferred_output(in, pbr_input);
#else
    let original_base = pbr_input.material.base_color;

    let diffuse_probe = max(
        apply_pbr_lighting(diffuse_probe_input(pbr_input)).rgb,
        vec3<f32>(0.0),
    );
    let full_probe = max(
        apply_pbr_lighting(full_probe_input(pbr_input)).rgb,
        vec3<f32>(0.0),
    );
    let specular_probe = max(full_probe - diffuse_probe, vec3<f32>(0.0));

    let diffuse_intensity = saturate_f32(luminance(diffuse_probe));
    let diffuse_sample = diffuse_shading(original_base.rgb, diffuse_probe, diffuse_intensity);

    let specular_metric = saturate_f32(
        luminance(specular_probe) / max(1.0 - toon.specular_width, 0.05),
    );
    let specular_band = smoothstep(
        toon.specular_threshold - toon.specular_softness,
        toon.specular_threshold + toon.specular_softness,
        specular_metric,
    ) * toon.specular_intensity;
    let specular_contribution =
        toon.specular_color.rgb * normalized_tint(specular_probe) * specular_band;

    let NdotV = saturate_f32(dot(normalize(pbr_input.N), normalize(pbr_input.V)));
    let rim_base = 1.0 - NdotV;
    var rim_band = smoothstep(
        toon.rim_threshold - toon.rim_softness,
        toon.rim_threshold + toon.rim_softness,
        rim_base,
    ) * toon.rim_intensity;
    if toon.rim_lit_only != 0u {
        rim_band *= diffuse_sample.a;
    }
    let rim_contribution = toon.rim_color.rgb * rim_band;

    let emissive_contribution =
        pbr_input.material.emissive.rgb * original_base.a *
        mix(1.0, view_bindings::view.exposure, pbr_input.material.emissive.a);

    var out: FragmentOutput;
    out.color = vec4<f32>(
        diffuse_sample.rgb + specular_contribution + rim_contribution + emissive_contribution,
        original_base.a,
    );
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
#endif
}
