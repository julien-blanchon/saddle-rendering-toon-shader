//! Preset showcase — displays each example-only toon look applied to the same
//! set of objects. Demonstrates the range of looks achievable from the neutral
//! builders without shipping style presets in the core crate.

use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use common::saddle_pane::prelude::*;
use saddle_rendering_toon_shader::{ToonMaterial, ToonShaderPlugin};

const PANE_TITLE: &str = "Presets";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.46, 0.52, 0.62)))
        .add_plugins((
            common::default_plugins("Toon Shader — Preset Showcase"),
            ToonShaderPlugin::default(),
        ))
        .add_plugins(common::pane_plugins())
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, cycle_presets));
    app.run();
}

#[derive(Component)]
struct PresetObject;

#[derive(Component)]
struct PresetLabel;

#[derive(Resource)]
struct CurrentPreset(usize);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 6.0, 16.0),
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
        "Preset Showcase\nUse the pane slider to cycle through example sample looks\nEach look is built from the crate's neutral tuning API",
    );

    let shapes: Vec<(&str, Handle<Mesh>, Color)> = vec![
        (
            "Sphere",
            meshes.add(Sphere::new(1.15).mesh().uv(48, 24)),
            Color::srgb(0.92, 0.78, 0.65),
        ),
        (
            "Torus",
            meshes.add(Torus::new(0.75, 0.28)),
            Color::srgb(0.85, 0.28, 0.22),
        ),
        (
            "Cube",
            meshes.add(Cuboid::new(1.3, 1.3, 1.3)),
            Color::srgb(0.38, 0.62, 0.82),
        ),
        (
            "Capsule",
            meshes.add(Capsule3d::new(0.45, 1.4)),
            Color::srgb(0.72, 0.82, 0.48),
        ),
        (
            "Cylinder",
            meshes.add(Cylinder::new(0.55, 1.8)),
            Color::srgb(0.88, 0.68, 0.42),
        ),
    ];

    let x_positions = [-5.6, -2.8, 0.0, 2.8, 5.6];
    let extension = common::sample_looks::anime_character();

    for (i, (name, mesh, color)) in shapes.into_iter().enumerate() {
        let x = x_positions[i];
        commands.spawn((
            Name::new(format!("{name} Showcase")),
            PresetObject,
            Mesh3d(mesh),
            MeshMaterial3d(
                toon_materials.add(extension.clone().material(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.35,
                    ..default()
                })),
            ),
            Transform::from_xyz(x, 1.3, 0.0).with_rotation(Quat::from_rotation_x(if i == 1 {
                0.8
            } else {
                0.0
            })),
            DemoSpin {
                axis: Vec3::new(0.0, 1.0, 0.15),
                speed: 0.3 + i as f32 * 0.06,
            },
        ));
    }

    // Preset name label
    commands.spawn((
        Name::new("Preset Name Label"),
        PresetLabel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: px(16.0),
            ..default()
        },
        Text::new(format!(
            "Preset: {}",
            common::sample_looks::SAMPLE_LOOK_NAMES[0]
        )),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.95, 0.7)),
    ));

    PaneBuilder::new(PANE_TITLE)
        .slider(
            "Preset",
            Slider::new(
                0.0..=(common::sample_looks::SAMPLE_LOOK_NAMES.len() - 1) as f64,
                0.0,
            )
            .step(1.0),
        )
        .at(PanePosition::TopRight)
        .spawn(&mut commands);

    commands.insert_resource(CurrentPreset(0));
}

fn cycle_presets(
    store: Res<PaneStore>,
    mut current: ResMut<CurrentPreset>,
    objects: Query<&MeshMaterial3d<ToonMaterial>, With<PresetObject>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut labels: Query<&mut Text, With<PresetLabel>>,
) {
    if !store.is_changed() {
        return;
    }

    let idx_f: f64 = store.get_or(PANE_TITLE, "Preset", 0.0);
    let idx = idx_f as usize;
    if idx == current.0 {
        return;
    }
    current.0 = idx;

    let extension = common::sample_looks::sample_look(idx);
    for handle in &objects {
        let Some(mat) = toon_materials.get_mut(handle) else {
            continue;
        };
        mat.extension = extension.clone();
    }

    let name = common::sample_looks::SAMPLE_LOOK_NAMES
        .get(idx)
        .unwrap_or(&"Custom");
    for mut text in &mut labels {
        text.0 = format!("Preset: {name}");
    }
}
