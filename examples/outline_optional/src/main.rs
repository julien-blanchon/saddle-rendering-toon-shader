use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use common::saddle_pane::prelude::*;
use saddle_rendering_outline::{OutlineHighlight, OutlinePlugin, OutlineStyle};
use saddle_rendering_toon_shader::{ToonMaterial, ToonShaderPlugin};

const PANE_TITLE: &str = "Outline Demo";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.045, 0.05, 0.065)))
        .add_plugins((
            common::default_plugins("Toon Shader - Optional Outline"),
            ToonShaderPlugin::default(),
            OutlinePlugin::default(),
        ))
        .add_plugins(common::pane_plugins())
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, sync_params));
    app.run();
}

#[derive(Component)]
struct OutlinedDemo;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 4.6, 11.0),
        Vec3::new(0.0, 1.2, -0.2),
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
        "Toon + Outline\nThis crate keeps outlines optional and separate\nUse the pane on the right to tune rim light and outline width",
    );

    commands.spawn((
        Name::new("Outlined Toon Mesh"),
        OutlinedDemo,
        Mesh3d(meshes.add(Sphere::new(1.25).mesh().uv(32, 18))),
        MeshMaterial3d(
            toon_materials.add(common::sample_looks::anime_character().material(
                StandardMaterial {
                    base_color: Color::srgb(0.86, 0.78, 0.68),
                    perceptual_roughness: 0.34,
                    ..default()
                },
            )),
        ),
        Transform::from_xyz(0.0, 1.35, -0.4),
        DemoSpin {
            axis: Vec3::new(0.0, 1.0, 0.15),
            speed: 0.46,
        },
        OutlineHighlight::default()
            .with_style(OutlineStyle::new(Color::srgb(1.0, 0.84, 0.24), 6.0)),
    ));

    PaneBuilder::new(PANE_TITLE)
        .slider("Rim Intensity", Slider::new(0.0..=2.0, 0.28).step(0.01))
        .slider("Outline Width", Slider::new(0.0..=16.0, 6.0).step(0.25))
        .at(PanePosition::TopRight)
        .spawn(&mut commands);
}

fn sync_params(
    store: Res<PaneStore>,
    mut demos: Query<(&MeshMaterial3d<ToonMaterial>, &mut OutlineHighlight), With<OutlinedDemo>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    if !store.is_changed() {
        return;
    }

    let rim_intensity: f64 = store.get_or(PANE_TITLE, "Rim Intensity", 0.28);
    let outline_width: f64 = store.get_or(PANE_TITLE, "Outline Width", 6.0);

    for (material_handle, mut outline) in &mut demos {
        if let Some(material) = toon_materials.get_mut(material_handle) {
            material.extension.rim.intensity = rim_intensity as f32;
        }
        outline.style.width = outline_width as f32;
    }
}
