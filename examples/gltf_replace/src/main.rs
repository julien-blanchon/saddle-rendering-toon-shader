//! glTF scene replacement — load a glTF model twice, apply toon shading to one
//! copy by simply attaching `ToonExtension` to the `SceneRoot`. Shows how easy
//! it is to toon-shade imported content.

use saddle_rendering_toon_shader_example_common as common;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use common::saddle_pane::prelude::*;
use saddle_rendering_toon_shader::{ToonExtension, ToonRim, ToonShaderPlugin};

const PANE_TITLE: &str = "Scene Toon";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.48, 0.54, 0.64)))
        .add_plugins((
            common::default_plugins("Toon Shader — glTF Replace"),
            ToonShaderPlugin::default(),
        ))
        .add_plugins(common::pane_plugins())
        .add_systems(Startup, setup)
        .add_systems(Update, sync_params);
    common::install_auto_exit(&mut app);
    app.run();
}

#[derive(Component)]
struct ToonScene;

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

    common::spawn_instructions(
        &mut commands,
        "glTF Scene Replacement\nLeft: Original PBR | Right: Toon Shaded\nJust add ToonExtension to the SceneRoot entity",
    );

    let flight_helmet = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"));

    // Original PBR version
    commands.spawn((
        Name::new("Original Flight Helmet"),
        SceneRoot(flight_helmet.clone()),
        Transform::from_xyz(0.8, 0.0, -0.7).with_scale(Vec3::splat(1.35)),
    ));

    // Toon version — just attach ToonExtension!
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
        ToonScene,
        SceneRoot(flight_helmet),
        Transform::from_xyz(4.6, 0.0, -0.7).with_scale(Vec3::splat(1.35)),
        ToonExtension::low_poly_prop()
            .with_ramp_texture(warm_ramp)
            .with_rim(
                ToonRim::default()
                    .with_intensity(0.15)
                    .with_threshold(0.5)
                    .with_softness(0.18),
            ),
    ));

    PaneBuilder::new(PANE_TITLE)
        .slider("Bands", Slider::new(2.0..=8.0, 4.0).step(1.0))
        .slider("Shadow Floor", Slider::new(0.0..=1.0, 0.24).step(0.01))
        .slider("Rim Intensity", Slider::new(0.0..=2.0, 0.15).step(0.01))
        .at(PanePosition::TopRight)
        .spawn(&mut commands);
}

fn sync_params(store: Res<PaneStore>, mut toon_scenes: Query<&mut ToonExtension, With<ToonScene>>) {
    if !store.is_changed() {
        return;
    }
    let bands: f64 = store.get_or(PANE_TITLE, "Bands", 4.0);
    let floor: f64 = store.get_or(PANE_TITLE, "Shadow Floor", 0.24);
    let rim: f64 = store.get_or(PANE_TITLE, "Rim Intensity", 0.15);

    for mut ext in &mut toon_scenes {
        ext.band_count = bands as u32;
        ext.shadow_floor = floor as f32;
        ext.rim.intensity = rim as f32;
    }
}
