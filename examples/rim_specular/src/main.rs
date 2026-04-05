//! Rim and specular showcase — three spheres with different toon styles showing
//! how rim lighting and specular bands interact. Live-tweakable via saddle-pane.

use saddle_rendering_toon_shader_example_common as common;

use bevy::prelude::*;
use common::DemoSpin;
use common::saddle_pane::prelude::*;
use saddle_rendering_toon_shader::{
    ToonExtension, ToonMaterial, ToonRim, ToonShaderPlugin, ToonSpecular,
};

const PANE_TITLE: &str = "Rim & Specular";

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.42, 0.48, 0.58)))
        .add_plugins((
            common::default_plugins("Toon Shader — Rim & Specular"),
            ToonShaderPlugin::default(),
        ))
        .add_plugins(common::pane_plugins())
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, sync_params));
    common::install_auto_exit(&mut app);
    app.run();
}

#[derive(Component)]
struct RimSpecularDemo;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 4.0, 12.0),
        Vec3::new(0.0, 1.4, 0.0),
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
        "Rim & Specular Showcase\nLeft: No effects | Center: Anime | Right: Glossy\nTweak rim/specular with the pane on the right",
    );

    let styles: Vec<(&str, ToonExtension, Color)> = vec![
        (
            "No Effects",
            ToonExtension::banded(3)
                .with_specular(ToonSpecular::default().with_intensity(0.0))
                .with_rim(ToonRim::default().with_intensity(0.0)),
            Color::srgb(0.8, 0.78, 0.72),
        ),
        (
            "Anime Style",
            ToonExtension::anime_character(),
            Color::srgb(0.92, 0.82, 0.68),
        ),
        (
            "Glossy Style",
            ToonExtension::glossy_vehicle(),
            Color::srgb(0.84, 0.26, 0.18),
        ),
    ];

    for (i, (label, extension, color)) in styles.into_iter().enumerate() {
        let x = -4.2 + i as f32 * 4.2;

        commands.spawn((
            Name::new(format!("{label} Sphere")),
            RimSpecularDemo,
            Mesh3d(meshes.add(Sphere::new(1.3).mesh().uv(48, 24))),
            MeshMaterial3d(toon_materials.add(extension.material(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.3,
                ..default()
            }))),
            Transform::from_xyz(x, 1.4, 0.0),
            DemoSpin {
                axis: Vec3::new(0.1, 1.0, 0.05),
                speed: 0.35 + i as f32 * 0.1,
            },
        ));
    }

    PaneBuilder::new(PANE_TITLE)
        .slider("Rim Intensity", Slider::new(0.0..=2.0, 0.28).step(0.01))
        .slider("Rim Threshold", Slider::new(0.0..=1.0, 0.55).step(0.01))
        .slider("Rim Softness", Slider::new(0.0..=1.0, 0.18).step(0.01))
        .slider("Specular Intensity", Slider::new(0.0..=2.0, 0.9).step(0.01))
        .slider(
            "Specular Threshold",
            Slider::new(0.0..=1.0, 0.54).step(0.01),
        )
        .slider("Specular Width", Slider::new(0.0..=1.0, 0.42).step(0.01))
        .at(PanePosition::TopRight)
        .spawn(&mut commands);
}

fn sync_params(
    store: Res<PaneStore>,
    demos: Query<&MeshMaterial3d<ToonMaterial>, With<RimSpecularDemo>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    if !store.is_changed() {
        return;
    }
    let rim_i: f64 = store.get_or(PANE_TITLE, "Rim Intensity", 0.28);
    let rim_t: f64 = store.get_or(PANE_TITLE, "Rim Threshold", 0.55);
    let rim_s: f64 = store.get_or(PANE_TITLE, "Rim Softness", 0.18);
    let spec_i: f64 = store.get_or(PANE_TITLE, "Specular Intensity", 0.9);
    let spec_t: f64 = store.get_or(PANE_TITLE, "Specular Threshold", 0.54);
    let spec_w: f64 = store.get_or(PANE_TITLE, "Specular Width", 0.42);

    for handle in &demos {
        let Some(mat) = toon_materials.get_mut(handle) else {
            continue;
        };
        mat.extension.rim.intensity = rim_i as f32;
        mat.extension.rim.threshold = rim_t as f32;
        mat.extension.rim.softness = rim_s as f32;
        mat.extension.specular.intensity = spec_i as f32;
        mat.extension.specular.threshold = spec_t as f32;
        mat.extension.specular.width = spec_w as f32;
    }
}
