use bevy::color::ColorToComponents;
use bevy::ecs::component::Component;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType};
use bevy::render::texture::GpuImage;
use bevy::shader::ShaderRef;

use crate::TOON_SHADER_HANDLE;

pub type ToonMaterial = ExtendedMaterial<StandardMaterial, ToonExtension>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub enum ToonDiffuseMode {
    #[default]
    Bands,
    RampTexture,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ToonSpecular {
    pub color: Color,
    pub threshold: f32,
    pub softness: f32,
    pub intensity: f32,
    pub width: f32,
}

impl ToonSpecular {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }

    pub fn disabled() -> Self {
        Self::default().with_intensity(0.0)
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_softness(mut self, softness: f32) -> Self {
        self.softness = softness;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn sanitized(&self) -> Self {
        Self {
            color: self.color,
            threshold: self.threshold.clamp(0.0, 1.0),
            softness: sanitize_unit_range(self.softness),
            intensity: sanitize_non_negative(self.intensity),
            width: sanitize_unit_range(self.width),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.intensity > 0.001
    }
}

impl Default for ToonSpecular {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            threshold: 0.58,
            softness: 0.08,
            intensity: 0.12,
            width: 0.22,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ToonRim {
    pub color: Color,
    pub threshold: f32,
    pub softness: f32,
    pub intensity: f32,
    pub lit_side_only: bool,
}

impl ToonRim {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }

    pub fn disabled() -> Self {
        Self::default().with_intensity(0.0)
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_softness(mut self, softness: f32) -> Self {
        self.softness = softness;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn lit_side_only(mut self, lit_side_only: bool) -> Self {
        self.lit_side_only = lit_side_only;
        self
    }

    pub fn sanitized(&self) -> Self {
        Self {
            color: self.color,
            threshold: self.threshold.clamp(0.0, 1.0),
            softness: sanitize_unit_range(self.softness),
            intensity: sanitize_non_negative(self.intensity),
            lit_side_only: self.lit_side_only,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.intensity > 0.001
    }
}

impl Default for ToonRim {
    fn default() -> Self {
        Self {
            color: Color::srgb(1.0, 0.97, 0.9),
            threshold: 0.62,
            softness: 0.12,
            intensity: 0.0,
            lit_side_only: true,
        }
    }
}

#[derive(Clone, ShaderType, Debug, PartialEq)]
pub struct ToonExtensionUniform {
    pub shadow_tint: Vec4,
    pub specular_color: Vec4,
    pub rim_color: Vec4,
    pub diffuse_mode: u32,
    pub band_count: u32,
    pub rim_lit_only: u32,
    pub _padding_mode: u32,
    pub band_softness: f32,
    pub shadow_floor: f32,
    pub light_wrap: f32,
    pub _padding_a: f32,
    pub specular_threshold: f32,
    pub specular_softness: f32,
    pub specular_intensity: f32,
    pub specular_width: f32,
    pub rim_threshold: f32,
    pub rim_softness: f32,
    pub rim_intensity: f32,
    pub _padding_b: f32,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, PartialEq, Component)]
#[uniform(100, ToonExtensionUniform)]
#[reflect(Component, Debug, PartialEq, Default)]
pub struct ToonExtension {
    pub diffuse_mode: ToonDiffuseMode,
    pub band_count: u32,
    pub band_softness: f32,
    pub shadow_floor: f32,
    pub shadow_tint: Color,
    pub light_wrap: f32,
    pub specular: ToonSpecular,
    pub rim: ToonRim,
    #[texture(101)]
    #[sampler(102)]
    pub ramp_texture: Option<Handle<Image>>,
}

impl ToonExtension {
    pub const MIN_BANDS: u32 = 2;
    pub const MAX_BANDS: u32 = 8;

    pub fn banded(band_count: u32) -> Self {
        Self::default().with_band_count(band_count)
    }

    pub fn ramped(ramp_texture: Handle<Image>) -> Self {
        Self::default().with_ramp_texture(ramp_texture)
    }

    pub fn with_band_count(mut self, band_count: u32) -> Self {
        self.band_count = band_count;
        self.diffuse_mode = ToonDiffuseMode::Bands;
        self
    }

    pub fn with_band_profile(mut self, band_count: u32, band_softness: f32) -> Self {
        self.band_count = band_count;
        self.band_softness = band_softness;
        self.diffuse_mode = ToonDiffuseMode::Bands;
        self
    }

    pub fn with_band_softness(mut self, band_softness: f32) -> Self {
        self.band_softness = band_softness;
        self
    }

    pub fn with_shadow_floor(mut self, shadow_floor: f32) -> Self {
        self.shadow_floor = shadow_floor;
        self
    }

    pub fn with_shadow_tint(mut self, shadow_tint: Color) -> Self {
        self.shadow_tint = shadow_tint;
        self
    }

    pub fn with_shadow_response(mut self, shadow_floor: f32, shadow_tint: Color) -> Self {
        self.shadow_floor = shadow_floor;
        self.shadow_tint = shadow_tint;
        self
    }

    pub fn with_light_wrap(mut self, light_wrap: f32) -> Self {
        self.light_wrap = light_wrap;
        self
    }

    pub fn with_specular(mut self, specular: ToonSpecular) -> Self {
        self.specular = specular;
        self
    }

    pub fn without_specular(self) -> Self {
        self.with_specular(ToonSpecular::disabled())
    }

    pub fn with_rim(mut self, rim: ToonRim) -> Self {
        self.rim = rim;
        self
    }

    pub fn without_rim(self) -> Self {
        self.with_rim(ToonRim::disabled())
    }

    pub fn with_ramp_texture(mut self, ramp_texture: Handle<Image>) -> Self {
        self.diffuse_mode = ToonDiffuseMode::RampTexture;
        self.ramp_texture = Some(ramp_texture);
        self
    }

    pub fn without_ramp_texture(mut self) -> Self {
        self.diffuse_mode = ToonDiffuseMode::Bands;
        self.ramp_texture = None;
        self
    }

    pub fn uses_ramp_texture(&self) -> bool {
        matches!(self.diffuse_mode, ToonDiffuseMode::RampTexture) && self.ramp_texture.is_some()
    }

    pub fn material(self, base: StandardMaterial) -> ToonMaterial {
        ToonMaterial {
            base,
            extension: self,
        }
    }

    pub(crate) fn sanitized_uniform(&self) -> ToonExtensionUniform {
        let specular = self.specular.sanitized();
        let rim = self.rim.sanitized();
        ToonExtensionUniform {
            shadow_tint: LinearRgba::from(self.shadow_tint).to_vec4(),
            specular_color: LinearRgba::from(specular.color).to_vec4(),
            rim_color: LinearRgba::from(rim.color).to_vec4(),
            diffuse_mode: match self.diffuse_mode {
                ToonDiffuseMode::Bands => 0,
                ToonDiffuseMode::RampTexture if self.ramp_texture.is_some() => 1,
                ToonDiffuseMode::RampTexture => 0,
            },
            band_count: self.band_count.clamp(Self::MIN_BANDS, Self::MAX_BANDS),
            rim_lit_only: u32::from(rim.lit_side_only),
            _padding_mode: 0,
            band_softness: sanitize_unit_range(self.band_softness),
            shadow_floor: sanitize_unit_range(self.shadow_floor),
            light_wrap: self.light_wrap.clamp(-0.5, 0.5),
            _padding_a: 0.0,
            specular_threshold: specular.threshold,
            specular_softness: specular.softness,
            specular_intensity: specular.intensity,
            specular_width: specular.width,
            rim_threshold: rim.threshold,
            rim_softness: rim.softness,
            rim_intensity: rim.intensity,
            _padding_b: 0.0,
        }
    }
}

impl Default for ToonExtension {
    fn default() -> Self {
        Self {
            diffuse_mode: ToonDiffuseMode::Bands,
            band_count: 2,
            band_softness: 0.12,
            shadow_floor: 0.16,
            shadow_tint: Color::srgb(0.22, 0.24, 0.3),
            light_wrap: 0.0,
            specular: ToonSpecular::default(),
            rim: ToonRim::default(),
            ramp_texture: None,
        }
    }
}

impl AsBindGroupShaderType<ToonExtensionUniform> for ToonExtension {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> ToonExtensionUniform {
        self.sanitized_uniform()
    }
}

impl MaterialExtension for ToonExtension {
    fn fragment_shader() -> ShaderRef {
        TOON_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        TOON_SHADER_HANDLE.into()
    }
}

pub(crate) fn build_toon_material(
    base: &StandardMaterial,
    extension: &ToonExtension,
) -> ToonMaterial {
    ToonMaterial {
        base: base.clone(),
        extension: extension.clone(),
    }
}

fn sanitize_unit_range(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn sanitize_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "material_tests.rs"]
mod material_tests;
