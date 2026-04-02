use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use saddle_rendering_toon_shader::{ToonExtension, ToonMaterial, ToonShaderPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.045, 0.05, 0.065)))
        .add_plugins((common::default_plugins("Toon Shader - Basic"), ToonShaderPlugin::default()))
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
    common::spawn_camera(&mut commands, Vec3::new(0.0, 4.8, 11.5), Vec3::new(0.0, 1.4, 0.0));
    common::spawn_lighting(&mut commands);
    common::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
    );

    commands.spawn((
        Name::new("Basic Toon Sphere"),
        Mesh3d(meshes.add(Sphere::new(1.35).mesh().uv(32, 18))),
        MeshMaterial3d(toon_materials.add(
            ToonExtension::low_poly_prop().material(StandardMaterial {
                base_color: Color::srgb(0.88, 0.74, 0.58),
                perceptual_roughness: 0.42,
                ..default()
            }),
        )),
        Transform::from_xyz(0.0, 1.45, 0.0),
        DemoSpin {
            axis: Vec3::new(0.0, 1.0, 0.2),
            speed: 0.4,
        },
    ));
}
