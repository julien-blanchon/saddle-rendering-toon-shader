use bevy::prelude::*;
use bevy::reflect::Reflect;

#[derive(Resource, Debug, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct ToonShaderDiagnostics {
    pub runtime_active: bool,
    pub managed_direct_entities: usize,
    pub managed_scene_entities: usize,
    pub scene_roots: usize,
    pub toon_material_assets: usize,
    pub ramp_materials: usize,
    pub rim_enabled_materials: usize,
    pub specular_enabled_materials: usize,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct ToonRuntimeState {
    pub active: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToonManagedMaterialSource {
    Direct,
    Scene,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToonManagedMaterial {
    pub source: ToonManagedMaterialSource,
}

impl ToonManagedMaterial {
    pub(crate) const fn direct() -> Self {
        Self {
            source: ToonManagedMaterialSource::Direct,
        }
    }

    pub(crate) const fn scene() -> Self {
        Self {
            source: ToonManagedMaterialSource::Scene,
        }
    }
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToonSceneReplacementComplete;
