use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::prelude::*;
use bevy::shader::Shader;

mod components;
mod material;
mod plugin;
mod scene_replace;
mod systems;
mod utils;

pub use components::ToonShaderDiagnostics;
pub use material::{ToonDiffuseMode, ToonExtension, ToonMaterial, ToonRim, ToonSpecular};
pub use plugin::{ToonShaderPlugin, ToonShaderSystems};

pub(crate) use material::build_toon_material;

pub(crate) const TOON_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("18d467fe-480a-4a2f-b509-e4ca3b0e58e2");

pub(crate) fn load_shader(app: &mut App) {
    load_internal_asset!(
        app,
        TOON_SHADER_HANDLE,
        "shaders/toon_shader.wgsl",
        Shader::from_wgsl
    );
}
