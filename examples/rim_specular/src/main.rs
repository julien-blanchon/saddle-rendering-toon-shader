use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use saddle_rendering_toon_shader::{
    ToonExtension, ToonMaterial, ToonRim, ToonShaderPlugin, ToonSpecular,
};

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.045, 0.06)))
        .add_plugins((
            common::default_plugins("Toon Shader - Rim And Specular"),
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
        Vec3::new(0.0, 4.8, 14.5),
        Vec3::new(0.0, 1.2, -0.6),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
    );

    let variations = [
        ToonExtension::banded(3)
            .with_specular(ToonSpecular::default().with_intensity(0.0))
            .with_rim(ToonRim::default().with_intensity(0.0)),
        ToonExtension::anime_character(),
        ToonExtension::glossy_vehicle(),
    ];
    let colors = [
        Color::srgb(0.8, 0.78, 0.72),
        Color::srgb(0.9, 0.84, 0.72),
        Color::srgb(0.84, 0.26, 0.18),
    ];

    for (index, (extension, color)) in variations.into_iter().zip(colors.into_iter()).enumerate() {
        commands.spawn((
            Name::new(format!("Rim Specular Demo {}", index + 1)),
            Mesh3d(meshes.add(Torus::new(0.8, 0.32))),
            MeshMaterial3d(toon_materials.add(extension.material(StandardMaterial {
                base_color: color,
                metallic: 0.02,
                perceptual_roughness: 0.28 + index as f32 * 0.12,
                ..default()
            }))),
            Transform::from_xyz(-4.6 + index as f32 * 4.6, 1.35, -0.9)
                .with_rotation(Quat::from_rotation_x(1.0)),
            DemoSpin {
                axis: Vec3::new(0.2, 1.0, 0.1),
                speed: 0.42 + index as f32 * 0.08,
            },
        ));
    }
}
