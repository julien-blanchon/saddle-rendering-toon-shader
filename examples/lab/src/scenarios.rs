use std::collections::HashSet;

use bevy::asset::AssetId;
use bevy::light::DirectionalLight;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_rendering_toon_shader::{
    ToonExtension, ToonMaterial, ToonRim, ToonShaderDiagnostics, ToonSpecular,
};

use crate::{
    GLOSSY_STAGE_POS, HERO_STAGE_POS, LabAssets, LabEntities, LabSunRig, NORMAL_STAGE_POS,
    STANDARD_HELMET_POS, TOON_HELMET_POS,
};

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "toon_shader_smoke" => Some(build_smoke()),
        "toon_shader_gltf_replace" => Some(build_gltf_replace()),
        "toon_shader_rim_specular" => Some(build_rim_specular()),
        _ => None,
    }
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "toon_shader_smoke",
        "toon_shader_gltf_replace",
        "toon_shader_rim_specular",
    ]
}

fn build_smoke() -> Scenario {
    Scenario::builder("toon_shader_smoke")
        .description(
            "Launch the lab, wait for the textured multi-mesh glTF showcase, then capture wide and shadow-stress verification shots.",
        )
        .then(Action::WaitUntil {
            label: "wait for multi-mesh gltf replacement".into(),
            condition: Box::new(gltf_showcase_ready),
            max_frames: 600,
        })
        .then(Action::WaitFrames(10))
        .then(assertions::resource_exists::<ToonShaderDiagnostics>(
            "toon diagnostics exists",
        ))
        .then(assertions::resource_satisfies::<ToonShaderDiagnostics>(
            "diagnostics show active managed content",
            |diagnostics| {
                diagnostics.runtime_active
                    && diagnostics.managed_direct_entities >= 3
                    && diagnostics.managed_scene_entities >= 5
                    && diagnostics.ramp_materials >= 1
            },
        ))
        .then(assertions::custom(
            "textured multi-mesh gltf replacement is visible on the toon root only",
            |world| gltf_showcase_ready(world) && gltf_roots_diverge(world),
        ))
        .then(inspect::log_resource::<ToonShaderDiagnostics>(
            "toon shader smoke diagnostics",
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            configure_direct_showcase(world);
            focus_camera_wide_showcase(world);
        })))
        .then(Action::WaitFrames(8))
        .then(Action::Screenshot("toon_shader_smoke_wide".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            configure_shadow_stress_showcase(world);
        })))
        .then(Action::WaitFrames(12))
        .then(Action::Screenshot("toon_shader_smoke_shadow_stress".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("toon_shader_smoke summary"))
        .build()
}

fn build_gltf_replace() -> Scenario {
    Scenario::builder("toon_shader_gltf_replace")
        .description(
            "Compare the untouched and toon-replaced textured FlightHelmet scenes, then mutate the root toon profile and confirm every descendant updates.",
        )
        .then(Action::WaitUntil {
            label: "wait for scene replacement".into(),
            condition: Box::new(gltf_showcase_ready),
            max_frames: 600,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            focus_camera_on_scenes(world);
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::custom(
            "toon scene descendants use toon materials across multiple meshes",
            |world| gltf_showcase_ready(world) && gltf_roots_diverge(world),
        ))
        .then(Action::Screenshot("toon_shader_gltf_compare".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let lab = *world.resource::<LabEntities>();
            let ramps = world.resource::<LabAssets>().clone();
            world.entity_mut(lab.toon_scene_root).insert(
                ToonExtension::anime_character().with_ramp_texture(ramps.pop_ramp.clone()),
            );
        })))
        .then(Action::Custom(Box::new(|world: &mut World| {
            focus_camera_on_gltf_update(world);
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "scene root sync updates every descendant toon material",
            |world| {
                let lab = *world.resource::<LabEntities>();
                let descendants = scene_toon_material_handles(world, lab.toon_scene_root);
                descendants.len() >= 5
                    && descendants.iter().all(|handle| {
                        world
                            .resource::<Assets<ToonMaterial>>()
                            .get(handle)
                            .is_some_and(|material| {
                                material.extension.band_count == 2
                                    && material.extension.uses_ramp_texture()
                                    && material.extension.rim.is_enabled()
                            })
                    })
            },
        ))
        .then(Action::Screenshot("toon_shader_gltf_updated".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("toon_shader_gltf_replace summary"))
        .build()
}

fn build_rim_specular() -> Scenario {
    Scenario::builder("toon_shader_rim_specular")
        .description(
            "Push the hero mesh into a strong rim/specular setup, verify the synced material values, and capture two lighting angles.",
        )
        .then(Action::WaitFrames(30))
        .then(Action::Custom(Box::new(|world: &mut World| {
            configure_rim_specular_showcase(world);
            focus_camera_on_rim_specular_front(world);
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "hero material receives synced rim and specular values",
            |world| {
                let lab = *world.resource::<LabEntities>();
                let Some(material_handle) = world.get::<MeshMaterial3d<ToonMaterial>>(lab.hero) else {
                    return false;
                };
                let Some(material) = world.resource::<Assets<ToonMaterial>>().get(material_handle) else {
                    return false;
                };
                material.extension.specular.intensity >= 0.95
                    && material.extension.rim.intensity >= 0.42
            },
        ))
        .then(Action::Screenshot("toon_shader_rim_specular_front".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            focus_camera_on_rim_specular_grazing(world);
        })))
        .then(Action::WaitFrames(18))
        .then(Action::Screenshot("toon_shader_rim_specular_grazing".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("toon_shader_rim_specular summary"))
        .build()
}

#[derive(Default)]
struct SceneMaterialBreakdown {
    standard_materials: usize,
    toon_materials: usize,
}

fn scene_material_breakdown(world: &World, root: Entity) -> SceneMaterialBreakdown {
    let mut breakdown = SceneMaterialBreakdown::default();

    for entity in descendants_of(world, root) {
        if world
            .get::<MeshMaterial3d<StandardMaterial>>(entity)
            .is_some()
        {
            breakdown.standard_materials += 1;
        }
        if world.get::<MeshMaterial3d<ToonMaterial>>(entity).is_some() {
            breakdown.toon_materials += 1;
        }
    }

    breakdown
}

fn scene_toon_material_handles(world: &World, root: Entity) -> Vec<Handle<ToonMaterial>> {
    let mut handles = Vec::new();
    let mut seen = HashSet::<AssetId<ToonMaterial>>::default();

    for entity in descendants_of(world, root) {
        let Some(handle) = world.get::<MeshMaterial3d<ToonMaterial>>(entity) else {
            continue;
        };
        if seen.insert(handle.id()) {
            handles.push(handle.0.clone());
        }
    }

    handles
}

fn gltf_showcase_ready(world: &World) -> bool {
    let lab = *world.resource::<LabEntities>();
    let standard = scene_material_breakdown(world, lab.standard_scene_root);
    let toon = scene_material_breakdown(world, lab.toon_scene_root);
    standard.standard_materials >= 5 && toon.toon_materials >= 5
}

fn gltf_roots_diverge(world: &World) -> bool {
    let lab = *world.resource::<LabEntities>();
    let standard = scene_material_breakdown(world, lab.standard_scene_root);
    let toon = scene_material_breakdown(world, lab.toon_scene_root);
    standard.standard_materials >= 5
        && standard.toon_materials == 0
        && toon.standard_materials == 0
        && toon.toon_materials >= 5
}

fn focus_camera_wide_showcase(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    let mut camera = world.entity_mut(lab.camera);
    let mut transform = camera.get_mut::<Transform>().expect("camera should exist");
    *transform =
        Transform::from_xyz(2.4, 7.8, 15.4).looking_at(Vec3::new(1.8, 2.8, -0.85), Vec3::Y);
}

fn focus_camera_on_scenes(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    let mut camera = world.entity_mut(lab.camera);
    let mut transform = camera.get_mut::<Transform>().expect("camera should exist");
    *transform =
        Transform::from_xyz(8.2, 3.7, 10.4).looking_at(Vec3::new(8.25, 1.95, -0.9), Vec3::Y);
}

fn focus_camera_on_gltf_update(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    let mut camera = world.entity_mut(lab.camera);
    let mut transform = camera.get_mut::<Transform>().expect("camera should exist");
    *transform = Transform::from_xyz(9.8, 3.2, 7.9)
        .looking_at(Vec3::new(TOON_HELMET_POS.x, 1.95, -0.9), Vec3::Y);
}

fn descendants_of(world: &World, root: Entity) -> Vec<Entity> {
    let mut descendants = Vec::new();
    let mut stack = vec![root];

    while let Some(entity) = stack.pop() {
        let Some(children) = world.entity(entity).get::<Children>() else {
            continue;
        };

        for child in children.iter() {
            descendants.push(child);
            stack.push(child);
        }
    }

    descendants
}

fn configure_rim_specular_showcase(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    world.resource_mut::<ClearColor>().0 = Color::srgb(0.56, 0.62, 0.72);

    world.entity_mut(lab.hero).insert(
        ToonExtension::anime_character()
            .with_shadow_floor(0.9)
            .with_shadow_tint(Color::srgb(0.56, 0.6, 0.68))
            .with_light_wrap(0.24)
            .with_specular(
                ToonSpecular::default()
                    .with_intensity(1.05)
                    .with_width(0.54)
                    .with_threshold(0.48),
            )
            .with_rim(
                ToonRim::default()
                    .with_intensity(0.46)
                    .with_threshold(0.46)
                    .with_softness(0.22),
            ),
    );

    world.entity_mut(lab.glossy).insert(
        ToonExtension::glossy_vehicle()
            .with_shadow_floor(0.7)
            .with_shadow_tint(Color::srgb(0.42, 0.34, 0.3)),
    );

    world.entity_mut(lab.normal_mapped).insert(
        ToonExtension::low_poly_prop()
            .with_shadow_floor(0.8)
            .with_shadow_tint(Color::srgb(0.5, 0.58, 0.66))
            .with_light_wrap(0.18),
    );

    world
        .entity_mut(lab.hero)
        .get_mut::<Transform>()
        .expect("hero mesh should exist")
        .translation = HERO_STAGE_POS;
    world
        .entity_mut(lab.glossy)
        .get_mut::<Transform>()
        .expect("glossy mesh should exist")
        .translation = GLOSSY_STAGE_POS;
    world
        .entity_mut(lab.normal_mapped)
        .get_mut::<Transform>()
        .expect("normal mapped mesh should exist")
        .translation = NORMAL_STAGE_POS;
    world
        .entity_mut(lab.standard_scene_root)
        .get_mut::<Transform>()
        .expect("standard scene root should exist")
        .translation = STANDARD_HELMET_POS + Vec3::new(4.8, 0.0, -0.95);
    world
        .entity_mut(lab.toon_scene_root)
        .get_mut::<Transform>()
        .expect("toon scene root should exist")
        .translation = TOON_HELMET_POS + Vec3::new(4.8, 0.0, -0.95);

    set_sun_transform(
        world,
        lab.sun,
        Vec3::new(-1.8, 9.4, 5.8),
        HERO_STAGE_POS + Vec3::new(0.2, -0.25, -0.2),
    );
}

fn configure_direct_showcase(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    world.resource_mut::<ClearColor>().0 = Color::srgb(0.6, 0.66, 0.74);
    if let Some(mut ambient) = world.get_resource_mut::<GlobalAmbientLight>() {
        ambient.brightness = 18.0;
        ambient.color = Color::srgb(0.86, 0.9, 1.0);
    }

    world.entity_mut(lab.hero).insert(
        ToonExtension::anime_character()
            .with_shadow_floor(0.86)
            .with_shadow_tint(Color::srgb(0.54, 0.56, 0.62)),
    );
    world.entity_mut(lab.glossy).insert(
        ToonExtension::glossy_vehicle()
            .with_light_wrap(0.14)
            .with_shadow_floor(0.68)
            .with_shadow_tint(Color::srgb(0.44, 0.34, 0.3)),
    );
    world.entity_mut(lab.normal_mapped).insert(
        ToonExtension::low_poly_prop()
            .with_shadow_floor(0.82)
            .with_shadow_tint(Color::srgb(0.46, 0.54, 0.62))
            .with_band_softness(0.24),
    );

    world
        .entity_mut(lab.hero)
        .get_mut::<Transform>()
        .expect("hero mesh should exist")
        .translation = HERO_STAGE_POS;
    world
        .entity_mut(lab.glossy)
        .get_mut::<Transform>()
        .expect("glossy mesh should exist")
        .translation = GLOSSY_STAGE_POS;
    world
        .entity_mut(lab.normal_mapped)
        .get_mut::<Transform>()
        .expect("normal mapped mesh should exist")
        .translation = NORMAL_STAGE_POS;
    world
        .entity_mut(lab.standard_scene_root)
        .get_mut::<Transform>()
        .expect("standard scene root should exist")
        .translation = STANDARD_HELMET_POS;
    world
        .entity_mut(lab.toon_scene_root)
        .get_mut::<Transform>()
        .expect("toon scene root should exist")
        .translation = TOON_HELMET_POS;

    set_sun_transform(
        world,
        lab.sun,
        Vec3::new(-2.8, 10.2, 6.1),
        Vec3::new(1.8, 1.7, -0.9),
    );
}

fn focus_camera_on_rim_specular_front(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    let mut camera = world.entity_mut(lab.camera);
    let mut transform = camera
        .get_mut::<Transform>()
        .expect("lab camera should exist");
    *transform = Transform::from_xyz(-1.3, 5.1, 9.1)
        .looking_at(HERO_STAGE_POS + Vec3::new(0.0, 0.95, -0.1), Vec3::Y);
}

fn focus_camera_on_rim_specular_grazing(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    let mut camera = world.entity_mut(lab.camera);
    let mut transform = camera
        .get_mut::<Transform>()
        .expect("lab camera should exist");
    *transform = Transform::from_xyz(-7.3, 4.2, 6.4)
        .looking_at(HERO_STAGE_POS + Vec3::new(0.0, 0.95, -0.1), Vec3::Y);
}

fn configure_shadow_stress_showcase(world: &mut World) {
    let lab = *world.resource::<LabEntities>();
    world.resource_mut::<ClearColor>().0 = Color::srgb(0.46, 0.53, 0.64);

    if let Some(mut ambient) = world.get_resource_mut::<GlobalAmbientLight>() {
        ambient.brightness = 4.0;
        ambient.color = Color::srgb(0.82, 0.88, 1.0);
    }

    if let Some(mut light) = world.entity_mut(lab.sun).get_mut::<DirectionalLight>() {
        light.illuminance = 52_000.0;
        light.shadows_enabled = true;
    }

    set_sun_transform(
        world,
        lab.sun,
        Vec3::new(9.4, 9.3, 1.3),
        Vec3::new(4.0, 1.7, -0.9),
    );

    let mut camera = world.entity_mut(lab.camera);
    let mut transform = camera
        .get_mut::<Transform>()
        .expect("lab camera should exist");
    *transform =
        Transform::from_xyz(5.4, 6.1, 10.8).looking_at(Vec3::new(4.1, 2.7, -0.95), Vec3::Y);
}

fn set_sun_transform(world: &mut World, sun: Entity, translation: Vec3, target: Vec3) {
    if let Some(mut rig) = world.get_resource_mut::<LabSunRig>() {
        rig.orbit = false;
        rig.translation = translation;
        rig.target = target;
    }

    let mut sun_entity = world.entity_mut(sun);
    let mut transform = sun_entity.get_mut::<Transform>().expect("sun should exist");
    *transform = Transform::from_translation(translation).looking_at(target, Vec3::Y);
}
