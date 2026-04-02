use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::scene::SceneRoot;
use bevy::shader::Shader;
use bevy::transform::TransformPlugin;

use crate::{
    ToonExtension, ToonMaterial, ToonShaderDiagnostics, ToonShaderPlugin, ToonShaderSystems,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin));
    app.init_asset::<Mesh>();
    app.init_asset::<Image>();
    app.init_asset::<Shader>();
    app.init_asset::<StandardMaterial>();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.add_plugins(ToonShaderPlugin::new(Activate, Deactivate, Tick));
    app.configure_sets(
        Tick,
        ToonShaderSystems::ReplaceSceneMaterials.before(ToonShaderSystems::SyncRuntimeParameters),
    );
    app
}

fn add_cube_mesh(app: &mut App) -> Mesh3d {
    let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
    Mesh3d(meshes.add(Cuboid::default()))
}

fn add_standard_material(app: &mut App, color: Color) -> Handle<StandardMaterial> {
    app.world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::from_color(color))
}

#[test]
fn plugin_registers_resources_and_starts_inactive_until_activated() {
    let mut app = test_app();
    assert!(app.world().contains_resource::<ToonShaderDiagnostics>());
    assert!(
        !app.world()
            .resource::<crate::components::ToonRuntimeState>()
            .active
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    assert!(
        app.world()
            .resource::<crate::components::ToonRuntimeState>()
            .active
    );
}

#[test]
fn direct_material_replacement_creates_toon_material_asset() {
    let mut app = test_app();
    app.world_mut().run_schedule(Activate);

    let mesh = add_cube_mesh(&mut app);
    let base_material = add_standard_material(&mut app, Color::srgb(0.3, 0.6, 0.9));

    let entity = app.world_mut().spawn((
        Name::new("Direct Toon Mesh"),
        mesh,
        MeshMaterial3d(base_material),
        ToonExtension::default(),
    ));
    let entity = entity.id();

    app.world_mut().run_schedule(Tick);
    app.world_mut().run_schedule(Tick);

    assert!(app.world().get::<MeshMaterial3d<StandardMaterial>>(entity).is_none());
    assert!(app.world().get::<MeshMaterial3d<ToonMaterial>>(entity).is_some());

    let diagnostics = app.world().resource::<ToonShaderDiagnostics>();
    assert_eq!(diagnostics.managed_direct_entities, 1);
    assert_eq!(diagnostics.toon_material_assets, 1);
}

#[test]
fn scene_replacement_reuses_one_toon_material_per_source_handle_and_stops() {
    let mut app = test_app();
    app.world_mut().run_schedule(Activate);

    let child_a_mesh = add_cube_mesh(&mut app);
    let child_b_mesh = add_cube_mesh(&mut app);
    let shared_material = add_standard_material(&mut app, Color::srgb(0.78, 0.82, 0.92));

    let root = app
        .world_mut()
        .spawn((
            Name::new("Scene Root"),
            SceneRoot(Handle::<Scene>::default()),
            ToonExtension::anime_character(),
        ))
        .id();

    let child_a = app
        .world_mut()
        .spawn((
            Name::new("Child A"),
            child_a_mesh,
            MeshMaterial3d(shared_material.clone()),
            ChildOf(root),
        ))
        .id();
    let child_b = app
        .world_mut()
        .spawn((
            Name::new("Child B"),
            child_b_mesh,
            MeshMaterial3d(shared_material),
            ChildOf(root),
        ))
        .id();

    app.world_mut().run_schedule(Tick);
    app.world_mut().run_schedule(Tick);

    let first_handle = app
        .world()
        .get::<MeshMaterial3d<ToonMaterial>>(child_a)
        .expect("child A should use a toon material")
        .clone();
    let second_handle = app
        .world()
        .get::<MeshMaterial3d<ToonMaterial>>(child_b)
        .expect("child B should use a toon material")
        .clone();

    assert_eq!(first_handle.id(), second_handle.id());
    assert!(app.world().get::<MeshMaterial3d<StandardMaterial>>(child_a).is_none());
    assert!(app.world().get::<MeshMaterial3d<StandardMaterial>>(child_b).is_none());
    assert!(
        app.world()
            .get::<crate::components::ToonSceneReplacementComplete>(root)
            .is_some()
    );

    let before_second_tick_assets = app.world().resource::<Assets<ToonMaterial>>().len();
    app.world_mut().run_schedule(Tick);
    let after_second_tick_assets = app.world().resource::<Assets<ToonMaterial>>().len();
    assert_eq!(before_second_tick_assets, after_second_tick_assets);

    let diagnostics = app.world().resource::<ToonShaderDiagnostics>();
    assert_eq!(diagnostics.scene_roots, 1);
    assert_eq!(diagnostics.managed_scene_entities, 2);
    assert_eq!(diagnostics.toon_material_assets, 1);
}

#[test]
fn local_override_blocks_root_sync_and_root_sync_updates_shared_descendants() {
    let mut app = test_app();
    app.world_mut().run_schedule(Activate);

    let base_material = add_standard_material(&mut app, Color::srgb(0.9, 0.7, 0.3));
    let inherited_mesh = add_cube_mesh(&mut app);
    let overridden_mesh = add_cube_mesh(&mut app);
    let root = app
        .world_mut()
        .spawn((
            Name::new("Root"),
            SceneRoot(Handle::<Scene>::default()),
            ToonExtension::low_poly_prop(),
        ))
        .id();

    let inherited = app
        .world_mut()
        .spawn((
            Name::new("Inherited"),
            inherited_mesh,
            MeshMaterial3d(base_material.clone()),
            ChildOf(root),
        ))
        .id();

    let overridden = app
        .world_mut()
        .spawn((
            Name::new("Overridden"),
            overridden_mesh,
            MeshMaterial3d(base_material),
            ChildOf(root),
            ToonExtension::glossy_vehicle(),
        ))
        .id();

    app.world_mut().run_schedule(Tick);
    app.world_mut().run_schedule(Tick);

    let override_handle = app
        .world()
        .get::<MeshMaterial3d<ToonMaterial>>(overridden)
        .expect("overridden child should use a toon material")
        .clone();

    app.world_mut()
        .entity_mut(root)
        .insert(ToonExtension::anime_character());

    app.world_mut().run_schedule(Tick);

    let inherited_handle = app
        .world()
        .get::<MeshMaterial3d<ToonMaterial>>(inherited)
        .expect("inherited child should still use a toon material")
        .clone();

    let inherited_material = app
        .world()
        .resource::<Assets<ToonMaterial>>()
        .get(&inherited_handle)
        .expect("inherited material should exist");
    let override_material = app
        .world()
        .resource::<Assets<ToonMaterial>>()
        .get(&override_handle)
        .expect("override material should exist");

    assert_eq!(inherited_material.extension.band_count, 2);
    assert_eq!(override_material.extension.band_count, 3);
}

#[test]
fn deactivation_stops_runtime_work_until_reactivated() {
    let mut app = test_app();
    let mesh = add_cube_mesh(&mut app);
    let material = add_standard_material(&mut app, Color::srgb(0.4, 0.4, 0.8));
    let entity = app
        .world_mut()
        .spawn((
            Name::new("Inactive Replacement"),
            mesh,
            MeshMaterial3d(material),
            ToonExtension::default(),
        ))
        .id();

    app.world_mut().run_schedule(Tick);
    assert!(app.world().get::<MeshMaterial3d<ToonMaterial>>(entity).is_none());

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);
    assert!(app.world().get::<MeshMaterial3d<ToonMaterial>>(entity).is_some());

    app.world_mut().run_schedule(Deactivate);
    app.world_mut().entity_mut(entity).insert(ToonExtension::anime_character());
    app.world_mut().run_schedule(Tick);

    let material_handle = app
        .world()
        .get::<MeshMaterial3d<ToonMaterial>>(entity)
        .expect("toon material should still exist")
        .clone();
    let extension = app
        .world()
        .resource::<Assets<ToonMaterial>>()
        .get(&material_handle)
        .expect("toon material asset should exist")
        .extension
        .clone();

    assert_eq!(extension.band_count, 2);
}
