//! Ramp texture showcase — compare discrete band shading with artist-authored
//! ramp textures. Each column uses a different ramp, demonstrating warm, cool,
//! and stylized color palettes.

use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use saddle_rendering_toon_shader::{ToonExtension, ToonMaterial, ToonShaderPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.42, 0.48, 0.56)))
        .add_plugins((
            common::default_plugins("Toon Shader — Ramp Textures"),
            ToonShaderPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, common::spin);
    common::install_auto_exit(&mut app);
    app.run();
}

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
            Mesh3d(meshes.add(Sphere::new(1.1).mesh().uv(48, 24))),
            MeshMaterial3d(
                toon_materials.add(
                    ToonExtension::default()
                        .with_ramp_texture(ramp)
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
            Mesh3d(meshes.add(Sphere::new(1.1).mesh().uv(48, 24))),
            MeshMaterial3d(
                toon_materials.add(
                    ToonExtension::banded(4)
                        .with_band_softness(0.08)
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
}
