use std::collections::HashSet;

use bevy::asset::AssetId;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

use crate::ToonExtension;
use crate::components::{
    ToonManagedMaterial, ToonManagedMaterialSource, ToonRuntimeState, ToonShaderDiagnostics,
};
use crate::material::ToonMaterial;
use crate::utils::iter_descendants;

pub(crate) fn activate_runtime(mut runtime: ResMut<ToonRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(mut runtime: ResMut<ToonRuntimeState>) {
    runtime.active = false;
}

pub(crate) fn runtime_is_active(runtime: Res<ToonRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn sync_runtime_parameters(
    direct_entities: Query<
        (&ToonExtension, &MeshMaterial3d<ToonMaterial>),
        (Changed<ToonExtension>, Without<SceneRoot>),
    >,
    scene_roots: Query<(Entity, &ToonExtension), (Changed<ToonExtension>, With<SceneRoot>)>,
    children: Query<&Children>,
    descendant_toon_materials: Query<
        (&MeshMaterial3d<ToonMaterial>, Option<&ToonExtension>),
        Without<SceneRoot>,
    >,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    for (extension, material_handle) in &direct_entities {
        let Some(material) = toon_materials.get_mut(material_handle) else {
            continue;
        };
        material.extension = extension.clone();
    }

    for (root, extension) in &scene_roots {
        let mut seen_handles = HashSet::<AssetId<ToonMaterial>>::default();

        for descendant in iter_descendants(root, &children) {
            let Ok((toon_handle, local_override)) = descendant_toon_materials.get(descendant)
            else {
                continue;
            };

            if local_override.is_some() {
                continue;
            }

            if !seen_handles.insert(toon_handle.id()) {
                continue;
            }

            let Some(material) = toon_materials.get_mut(toon_handle) else {
                continue;
            };
            material.extension = extension.clone();
        }
    }
}

pub(crate) fn publish_diagnostics(
    runtime: Res<ToonRuntimeState>,
    managed_materials: Query<&ToonManagedMaterial>,
    scene_roots: Query<Entity, (With<SceneRoot>, With<ToonExtension>)>,
    toon_materials: Res<Assets<ToonMaterial>>,
    mut diagnostics: ResMut<ToonShaderDiagnostics>,
) {
    let mut managed_direct_entities = 0usize;
    let mut managed_scene_entities = 0usize;

    for managed in &managed_materials {
        match managed.source {
            ToonManagedMaterialSource::Direct => managed_direct_entities += 1,
            ToonManagedMaterialSource::Scene => managed_scene_entities += 1,
        }
    }

    let mut ramp_materials = 0usize;
    let mut rim_enabled_materials = 0usize;
    let mut specular_enabled_materials = 0usize;

    for (_, material) in toon_materials.iter() {
        if material.extension.uses_ramp_texture() {
            ramp_materials += 1;
        }
        if material.extension.rim.is_enabled() {
            rim_enabled_materials += 1;
        }
        if material.extension.specular.is_enabled() {
            specular_enabled_materials += 1;
        }
    }

    *diagnostics = ToonShaderDiagnostics {
        runtime_active: runtime.active,
        managed_direct_entities,
        managed_scene_entities,
        scene_roots: scene_roots.iter().count(),
        toon_material_assets: toon_materials.len(),
        ramp_materials,
        rim_enabled_materials,
        specular_enabled_materials,
    };
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
