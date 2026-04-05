//! Basic toon shader example — side-by-side PBR vs Toon comparison.
//!
//! The left group uses standard PBR materials, the right group uses the same
//! base colors with `ToonExtension` applied. Demonstrates that adding toon
//! shading is just swapping the material type.

use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use common::saddle_pane::prelude::*;
use saddle_rendering_toon_shader::{ToonExtension, ToonMaterial, ToonShaderPlugin};

const PANE_TITLE: &str = "Toon Parameters";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.52, 0.58, 0.68)))
        .add_plugins(common::default_plugins("Toon Shader — PBR vs Toon"))
        .add_plugins(ToonShaderPlugin::default())
        .add_plugins(common::pane_plugins())
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, sync_toon_params));
    common::install_auto_exit(&mut app);
    app.run();
}

#[derive(Component)]
struct ToonObject;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 5.5, 14.0),
        Vec3::new(0.0, 1.5, 0.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
    );

    common::spawn_instructions(
        &mut commands,
        "PBR vs Toon Shader comparison\nBack row: Standard PBR  |  Front row: Toon Shaded\nUse the pane on the right to tweak toon parameters",
    );

    let shapes: Vec<(&str, Handle<Mesh>, Color)> = vec![
        (
            "Sphere",
            meshes.add(Sphere::new(1.0).mesh().uv(48, 24)),
            Color::srgb(0.92, 0.78, 0.65),
        ),
        (
            "Torus",
            meshes.add(Torus::new(0.7, 0.28)),
            Color::srgb(0.85, 0.28, 0.22),
        ),
        (
            "Cube",
            meshes.add(Cuboid::new(1.3, 1.3, 1.3)),
            Color::srgb(0.38, 0.62, 0.82),
        ),
    ];

    let x_positions = [-2.8, 0.0, 2.8];
    let pbr_z = 2.0;
    let toon_z = -2.0;

    let extension = ToonExtension::anime_character();

    for (i, (name, mesh, color)) in shapes.into_iter().enumerate() {
        let x = x_positions[i];
        let base_mat = StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.35,
            ..default()
        };

        // PBR version (back row)
        commands.spawn((
            Name::new(format!("{name} PBR")),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(standard_materials.add(base_mat.clone())),
            Transform::from_xyz(x, 1.2, pbr_z).with_rotation(Quat::from_rotation_x(if i == 1 {
                0.8
            } else {
                0.0
            })),
            DemoSpin {
                axis: Vec3::new(0.0, 1.0, 0.15),
                speed: 0.3 + i as f32 * 0.08,
            },
        ));

        // Toon version (front row)
        commands.spawn((
            Name::new(format!("{name} Toon")),
            ToonObject,
            Mesh3d(mesh),
            MeshMaterial3d(toon_materials.add(extension.clone().material(base_mat))),
            Transform::from_xyz(x, 1.2, toon_z).with_rotation(Quat::from_rotation_x(if i == 1 {
                0.8
            } else {
                0.0
            })),
            DemoSpin {
                axis: Vec3::new(0.0, 1.0, 0.15),
                speed: 0.3 + i as f32 * 0.08,
            },
        ));
    }

    PaneBuilder::new(PANE_TITLE)
        .slider("Bands", Slider::new(2.0..=8.0, 2.0).step(1.0))
        .slider("Band Softness", Slider::new(0.0..=1.0, 0.12).step(0.01))
        .slider("Shadow Floor", Slider::new(0.0..=1.0, 0.12).step(0.01))
        .slider("Rim Intensity", Slider::new(0.0..=2.0, 0.28).step(0.01))
        .slider("Specular Intensity", Slider::new(0.0..=2.0, 0.9).step(0.01))
        .at(PanePosition::TopRight)
        .spawn(&mut commands);
}

fn sync_toon_params(
    store: Res<PaneStore>,
    toon_objects: Query<&MeshMaterial3d<ToonMaterial>, With<ToonObject>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    if !store.is_changed() {
        return;
    }
    let bands: f64 = store.get_or(PANE_TITLE, "Bands", 2.0);
    let softness: f64 = store.get_or(PANE_TITLE, "Band Softness", 0.12);
    let floor: f64 = store.get_or(PANE_TITLE, "Shadow Floor", 0.12);
    let rim: f64 = store.get_or(PANE_TITLE, "Rim Intensity", 0.28);
    let spec: f64 = store.get_or(PANE_TITLE, "Specular Intensity", 0.9);

    for handle in &toon_objects {
        let Some(mat) = toon_materials.get_mut(handle) else {
            continue;
        };
        mat.extension.band_count = bands as u32;
        mat.extension.band_softness = softness as f32;
        mat.extension.shadow_floor = floor as f32;
        mat.extension.rim.intensity = rim as f32;
        mat.extension.specular.intensity = spec as f32;
    }
}
