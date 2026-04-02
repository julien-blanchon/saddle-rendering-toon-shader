use saddle_rendering_toon_shader_example_common as common;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use saddle_rendering_toon_shader::ToonShaderPlugin;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.045, 0.05, 0.065)))
        .add_plugins((
            common::default_plugins("Toon Shader - glTF Replace"),
            ToonShaderPlugin::default(),
        ))
        .add_systems(Startup, setup);
    common::install_auto_exit(&mut app);
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(3.4, 2.6, 10.2),
        Vec3::new(2.8, 0.8, -0.3),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
    );

    let flight_helmet = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"),
    );

    commands.spawn((
        Name::new("Original Flight Helmet"),
        SceneRoot(flight_helmet.clone()),
        Transform::from_xyz(0.8, 0.0, -0.7).with_scale(Vec3::splat(1.35)),
    ));

    let warm_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.12, 0.1, 0.18),
            Color::srgb(0.42, 0.32, 0.26),
            Color::srgb(0.86, 0.72, 0.5),
            Color::srgb(1.0, 0.94, 0.78),
        ],
    );

    commands.spawn((
        Name::new("Toon Flight Helmet"),
        SceneRoot(flight_helmet),
        Transform::from_xyz(4.6, 0.0, -0.7).with_scale(Vec3::splat(1.35)),
        saddle_rendering_toon_shader::ToonExtension::low_poly_prop().with_ramp_texture(warm_ramp),
    ));
}
