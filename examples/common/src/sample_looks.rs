use bevy::prelude::*;
use saddle_rendering_toon_shader::{ToonExtension, ToonRim, ToonSpecular};

pub const SAMPLE_LOOK_NAMES: &[&str] = &[
    "Anime Character",
    "Low Poly Prop",
    "Glossy Vehicle",
    "Wind Waker",
    "Borderlands",
    "Flat Cel",
];

pub fn sample_look(index: usize) -> ToonExtension {
    match index {
        0 => anime_character(),
        1 => low_poly_prop(),
        2 => glossy_vehicle(),
        3 => wind_waker(),
        4 => borderlands(),
        5 => flat_cel(),
        _ => ToonExtension::default(),
    }
}

pub fn anime_character() -> ToonExtension {
    ToonExtension::banded(2)
        .with_shadow_response(0.12, Color::srgb(0.43, 0.48, 0.7))
        .with_specular(
            ToonSpecular::default()
                .with_intensity(0.9)
                .with_width(0.42)
                .with_threshold(0.54),
        )
        .with_rim(
            ToonRim::default()
                .with_intensity(0.28)
                .with_threshold(0.55)
                .with_softness(0.18),
        )
}

pub fn low_poly_prop() -> ToonExtension {
    ToonExtension::default()
        .with_band_profile(4, 0.18)
        .with_shadow_response(0.24, Color::srgb(0.26, 0.3, 0.38))
        .without_specular()
        .with_rim(ToonRim::default().with_intensity(0.06))
}

pub fn glossy_vehicle() -> ToonExtension {
    ToonExtension::banded(3)
        .with_shadow_response(0.14, Color::srgb(0.16, 0.2, 0.28))
        .with_light_wrap(0.08)
        .with_specular(
            ToonSpecular::new(Color::srgb(1.0, 0.98, 0.9))
                .with_intensity(1.1)
                .with_width(0.58)
                .with_threshold(0.48)
                .with_softness(0.1),
        )
        .with_rim(
            ToonRim::new(Color::srgb(1.0, 0.95, 0.85))
                .with_intensity(0.22)
                .with_threshold(0.5)
                .with_softness(0.16),
        )
}

pub fn wind_waker() -> ToonExtension {
    ToonExtension::banded(2)
        .with_band_softness(0.06)
        .with_shadow_response(0.22, Color::srgb(0.45, 0.38, 0.55))
        .with_light_wrap(0.04)
        .without_specular()
        .with_rim(
            ToonRim::new(Color::srgb(1.0, 0.96, 0.88))
                .with_intensity(0.2)
                .with_threshold(0.6)
                .with_softness(0.15),
        )
}

pub fn borderlands() -> ToonExtension {
    ToonExtension::banded(3)
        .with_band_softness(0.0)
        .with_shadow_response(0.05, Color::srgb(0.12, 0.1, 0.14))
        .with_light_wrap(-0.05)
        .without_specular()
        .without_rim()
}

pub fn flat_cel() -> ToonExtension {
    ToonExtension::banded(2)
        .with_band_softness(0.0)
        .with_shadow_response(0.08, Color::srgb(0.3, 0.28, 0.36))
        .without_specular()
        .without_rim()
}
