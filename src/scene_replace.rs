use std::collections::HashMap;

use bevy::asset::AssetId;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

use crate::components::{ToonManagedMaterial, ToonSceneReplacementComplete};
use crate::utils::iter_descendants;
use crate::{ToonExtension, ToonMaterial, build_toon_material};

pub(crate) fn clear_completed_scene_roots(
    changed_roots: Query<Entity, (Changed<SceneRoot>, With<ToonSceneReplacementComplete>)>,
    mut commands: Commands,
) {
    for root in &changed_roots {
        commands
            .entity(root)
            .remove::<ToonSceneReplacementComplete>();
    }
}

pub(crate) fn replace_direct_materials(
    direct_entities: Query<
        (Entity, &ToonExtension, &MeshMaterial3d<StandardMaterial>),
        Without<SceneRoot>,
    >,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut commands: Commands,
) {
    for (entity, extension, standard_handle) in &direct_entities {
        let Some(base_material) = standard_materials.get(standard_handle) else {
            continue;
        };
        let toon_handle = toon_materials.add(build_toon_material(base_material, extension));
        commands
            .entity(entity)
            .insert((MeshMaterial3d(toon_handle), ToonManagedMaterial::direct()));
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

pub(crate) fn replace_scene_materials(
    scene_roots: Query<
        (Entity, &ToonExtension),
        (With<SceneRoot>, Without<ToonSceneReplacementComplete>),
    >,
    children: Query<&Children>,
    descendant_materials: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&ToonExtension>,
        ),
        Without<SceneRoot>,
    >,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut commands: Commands,
) {
    for (root, extension) in &scene_roots {
        let mut converted: HashMap<AssetId<StandardMaterial>, Handle<ToonMaterial>> =
            HashMap::default();
        let mut saw_descendants = false;
        let mut unresolved_material = false;

        for descendant in iter_descendants(root, &children) {
            saw_descendants = true;
            let Ok((entity, standard_handle, local_override)) =
                descendant_materials.get(descendant)
            else {
                continue;
            };

            if local_override.is_some() {
                continue;
            }

            let Some(base_material) = standard_materials.get(standard_handle) else {
                unresolved_material = true;
                continue;
            };

            let toon_handle = converted
                .entry(standard_handle.id())
                .or_insert_with(|| {
                    toon_materials.add(build_toon_material(base_material, extension))
                })
                .clone();

            commands
                .entity(entity)
                .insert((MeshMaterial3d(toon_handle), ToonManagedMaterial::scene()));
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }

        // Once descendants exist and every root-managed StandardMaterial has been
        // resolved, mark the root as complete so later frames stop rescanning it.
        if saw_descendants && !unresolved_material {
            commands.entity(root).insert(ToonSceneReplacementComplete);
        }
    }
}
