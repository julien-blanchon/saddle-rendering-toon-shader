//! Ramp texture showcase — compare discrete band shading with artist-authored
//! ramp textures. Each column uses a different ramp, demonstrating warm, cool,
//! and stylized color palettes.

use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use common::saddle_pane::prelude::*;
use saddle_rendering_toon_shader::{ToonExtension, ToonMaterial, ToonShaderPlugin};

const PANE_TITLE: &str = "Ramp Comparison";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.42, 0.48, 0.56)))
        .add_plugins((
            common::default_plugins("Toon Shader — Ramp Textures"),
            ToonShaderPlugin::default(),
        ))
        .add_plugins(common::pane_plugins())
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, sync_params));
    app.run();
}

#[derive(Component)]
struct RampDemoObject;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 5.0, 14.0),
        Vec3::new(0.0, 1.2, 0.0),
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
        "Ramp Textures vs Discrete Bands\nTop row: ramp textures | Bottom row: 4-band discrete\nSame base colors, different shading styles",
    );

    let neutral_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.12, 0.13, 0.16),
            Color::srgb(0.42, 0.45, 0.56),
            Color::srgb(0.78, 0.8, 0.88),
            Color::srgb(1.0, 1.0, 1.0),
        ],
    );
    let warm_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.16, 0.12, 0.18),
            Color::srgb(0.52, 0.36, 0.28),
            Color::srgb(0.88, 0.68, 0.44),
            Color::srgb(1.0, 0.92, 0.72),
        ],
    );
    let pop_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.08, 0.06, 0.14),
            Color::srgb(0.22, 0.2, 0.62),
            Color::srgb(0.62, 0.42, 0.92),
            Color::srgb(1.0, 0.74, 0.34),
        ],
    );

    let x_positions = [-4.2, 0.0, 4.2];
    let ramps = [neutral_ramp, warm_ramp, pop_ramp];
    let labels = ["Neutral Ramp", "Warm Ramp", "Pop Ramp"];
    let colors = [
        Color::srgb(0.88, 0.82, 0.74),
        Color::srgb(0.76, 0.86, 0.96),
        Color::srgb(0.92, 0.78, 0.7),
    ];

    for (i, ((x, ramp), color)) in x_positions
        .into_iter()
        .zip(ramps.into_iter())
        .zip(colors.into_iter())
        .enumerate()
    {
        let base_mat = StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.42,
            ..default()
        };

        // Top row: ramp texture version
        commands.spawn((
            Name::new(format!("{} Sphere", labels[i])),
            RampDemoObject,
            Mesh3d(meshes.add(Sphere::new(1.1).mesh().uv(48, 24))),
            MeshMaterial3d(
                toon_materials.add(
                    ToonExtension::ramped(ramp)
                        .with_shadow_floor(0.18)
                        .material(base_mat.clone()),
                ),
            ),
            Transform::from_xyz(x, 2.6, -1.5),
            DemoSpin {
                axis: Vec3::new(0.1, 1.0, 0.0),
                speed: 0.32 + i as f32 * 0.06,
            },
        ));

        // Bottom row: discrete band version (same color)
        commands.spawn((
            Name::new(format!("{} Banded Sphere", labels[i])),
            RampDemoObject,
            Mesh3d(meshes.add(Sphere::new(1.1).mesh().uv(48, 24))),
            MeshMaterial3d(
                toon_materials.add(
                    ToonExtension::default()
                        .with_band_profile(4, 0.08)
                        .with_shadow_floor(0.18)
                        .material(base_mat),
                ),
            ),
            Transform::from_xyz(x, 1.15, 1.5),
            DemoSpin {
                axis: Vec3::new(0.1, 1.0, 0.0),
                speed: 0.32 + i as f32 * 0.06,
            },
        ));
    }

    PaneBuilder::new(PANE_TITLE)
        .slider("Bands", Slider::new(2.0..=8.0, 4.0).step(1.0))
        .slider("Band Softness", Slider::new(0.0..=1.0, 0.08).step(0.01))
        .slider("Shadow Floor", Slider::new(0.0..=1.0, 0.18).step(0.01))
        .at(PanePosition::TopRight)
        .spawn(&mut commands);
}

fn sync_params(
    store: Res<PaneStore>,
    demos: Query<&MeshMaterial3d<ToonMaterial>, With<RampDemoObject>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    if !store.is_changed() {
        return;
    }

    let bands: f64 = store.get_or(PANE_TITLE, "Bands", 4.0);
    let softness: f64 = store.get_or(PANE_TITLE, "Band Softness", 0.08);
    let shadow_floor: f64 = store.get_or(PANE_TITLE, "Shadow Floor", 0.18);

    for handle in &demos {
        let Some(material) = toon_materials.get_mut(handle) else {
            continue;
        };
        material.extension.band_count = bands as u32;
        material.extension.band_softness = softness as f32;
        material.extension.shadow_floor = shadow_floor as f32;
    }
}
