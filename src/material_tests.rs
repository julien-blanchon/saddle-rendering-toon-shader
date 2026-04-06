use bevy::prelude::*;

use super::*;

#[test]
fn default_extension_uses_two_band_mode_without_ramp() {
    let extension = ToonExtension::default();

    assert_eq!(extension.diffuse_mode, ToonDiffuseMode::Bands);
    assert_eq!(extension.band_count, 2);
    assert!(!extension.uses_ramp_texture());
}

#[test]
fn low_level_builders_cover_common_tuning_paths() {
    let ramped = ToonExtension::ramped(Handle::<Image>::default());
    assert_eq!(ramped.diffuse_mode, ToonDiffuseMode::RampTexture);
    assert!(ramped.uses_ramp_texture());

    let banded = ToonExtension::default()
        .with_band_profile(4, 0.18)
        .with_shadow_response(0.24, Color::srgb(0.26, 0.3, 0.38))
        .without_specular()
        .with_rim(ToonRim::default().with_intensity(0.06));
    assert_eq!(banded.band_count, 4);
    assert_eq!(banded.band_softness, 0.18);
    assert_eq!(banded.shadow_floor, 0.24);
    assert!(!banded.specular.is_enabled());
    assert!(banded.rim.is_enabled());

    let hard_edge = ToonExtension::banded(2)
        .with_band_profile(2, 0.0)
        .without_rim();
    assert_eq!(hard_edge.band_softness, 0.0);
    assert!(!hard_edge.rim.is_enabled());
}

#[test]
fn sanitized_uniform_clamps_invalid_numeric_ranges() {
    let extension = ToonExtension {
        band_count: 99,
        band_softness: f32::NAN,
        shadow_floor: 42.0,
        light_wrap: -8.0,
        specular: ToonSpecular::default()
            .with_threshold(-2.0)
            .with_softness(9.0)
            .with_intensity(-5.0)
            .with_width(7.0),
        rim: ToonRim::default()
            .with_threshold(8.0)
            .with_softness(f32::INFINITY)
            .with_intensity(-3.0),
        ..Default::default()
    };

    let uniform = extension.sanitized_uniform();
    assert_eq!(uniform.band_count, 8); // MAX_BANDS
    assert_eq!(uniform.band_softness, 0.0);
    assert_eq!(uniform.shadow_floor, 1.0);
    assert_eq!(uniform.light_wrap, -0.5);
    assert_eq!(uniform.specular_threshold, 0.0);
    assert_eq!(uniform.specular_softness, 1.0);
    assert_eq!(uniform.specular_intensity, 0.0);
    assert_eq!(uniform.specular_width, 1.0);
    assert_eq!(uniform.rim_threshold, 1.0);
    assert_eq!(uniform.rim_softness, 0.0);
    assert_eq!(uniform.rim_intensity, 0.0);
}

#[test]
fn ramp_mode_without_texture_falls_back_to_band_mode_in_uniform() {
    let extension = ToonExtension {
        diffuse_mode: ToonDiffuseMode::RampTexture,
        ramp_texture: None,
        ..Default::default()
    };

    assert_eq!(extension.sanitized_uniform().diffuse_mode, 0);
}

#[test]
fn disabled_feature_helpers_clear_their_respective_channels() {
    assert!(!ToonSpecular::disabled().is_enabled());
    assert!(!ToonRim::disabled().is_enabled());
}

#[test]
fn material_helper_preserves_base_rendering_method() {
    for render_method in [
        bevy::pbr::OpaqueRendererMethod::Auto,
        bevy::pbr::OpaqueRendererMethod::Forward,
        bevy::pbr::OpaqueRendererMethod::Deferred,
    ] {
        let material = ToonExtension::default().material(StandardMaterial {
            opaque_render_method: render_method,
            ..default()
        });
        assert_eq!(material.base.opaque_render_method, render_method);
    }
}

#[test]
fn cloned_material_keeps_original_rendering_method() {
    for render_method in [
        bevy::pbr::OpaqueRendererMethod::Auto,
        bevy::pbr::OpaqueRendererMethod::Forward,
        bevy::pbr::OpaqueRendererMethod::Deferred,
    ] {
        let material = build_toon_material(
            &StandardMaterial {
                opaque_render_method: render_method,
                ..default()
            },
            &ToonExtension::default(),
        );
        assert_eq!(material.base.opaque_render_method, render_method);
    }
}
