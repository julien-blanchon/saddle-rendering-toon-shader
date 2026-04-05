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
fn preset_builders_enable_expected_features() {
    let anime = ToonExtension::anime_character();
    assert!(anime.specular.is_enabled());
    assert!(anime.rim.is_enabled());

    let prop = ToonExtension::low_poly_prop();
    assert!(!prop.specular.is_enabled());
    assert!(prop.band_count >= 3);

    let vehicle = ToonExtension::glossy_vehicle();
    assert!(vehicle.specular.is_enabled());
    assert!(vehicle.rim.is_enabled());

    let ww = ToonExtension::wind_waker();
    assert_eq!(ww.band_count, 2);
    assert!(!ww.specular.is_enabled());
    assert!(ww.rim.is_enabled());

    let bl = ToonExtension::borderlands();
    assert_eq!(bl.band_count, 3);
    assert_eq!(bl.band_softness, 0.0);
    assert!(!bl.specular.is_enabled());
    assert!(!bl.rim.is_enabled());

    let flat = ToonExtension::flat_cel();
    assert_eq!(flat.band_count, 2);
    assert_eq!(flat.band_softness, 0.0);
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
fn material_helper_forces_forward_rendering() {
    let material = ToonExtension::default().material(StandardMaterial::default());
    assert_eq!(
        material.base.opaque_render_method,
        bevy::pbr::OpaqueRendererMethod::Forward
    );
}
