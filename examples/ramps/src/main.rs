use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use saddle_rendering_toon_shader::{ToonExtension, ToonMaterial, ToonShaderPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.055, 0.07)))
        .add_plugins((common::default_plugins("Toon Shader - Ramps"), ToonShaderPlugin::default()))
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
    common::spawn_camera(&mut commands, Vec3::new(0.0, 5.0, 14.0), Vec3::new(0.0, 1.2, 0.0));
    common::spawn_lighting(&mut commands);
    common::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
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

    let positions = [-4.2, 0.0, 4.2];
    let ramps = [neutral_ramp, warm_ramp, pop_ramp];
    let colors = [
        Color::srgb(0.88, 0.82, 0.74),
        Color::srgb(0.76, 0.86, 0.96),
        Color::srgb(0.92, 0.78, 0.7),
    ];

    for (index, ((x, ramp), color)) in positions
        .into_iter()
        .zip(ramps.into_iter())
        .zip(colors.into_iter())
        .enumerate()
    {
        commands.spawn((
            Name::new(format!("Ramp Demo {}", index + 1)),
            Mesh3d(meshes.add(Sphere::new(1.2).mesh().uv(32, 18))),
            MeshMaterial3d(toon_materials.add(
                ToonExtension::default().with_ramp_texture(ramp).material(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.42,
                    ..default()
                }),
            )),
            Transform::from_xyz(x, 1.25, 0.0),
            DemoSpin {
                axis: Vec3::new(0.1, 1.0, 0.0),
                speed: 0.32 + index as f32 * 0.06,
            },
        ));
    }
}
