#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_rendering_toon_shader_example_common as common;

use std::fmt::Write as _;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use common::DemoSpin;
use saddle_rendering_toon_shader::{ToonExtension, ToonShaderDiagnostics, ToonShaderPlugin};

const DEFAULT_BRP_PORT: u16 = 15_744;
const LAB_EXIT_ENV: &str = "TOON_SHADER_LAB_EXIT_AFTER_SECONDS";
pub(crate) const HERO_STAGE_POS: Vec3 = Vec3::new(-4.2, 2.0, -0.55);
pub(crate) const GLOSSY_STAGE_POS: Vec3 = Vec3::new(-0.35, 1.78, -1.05);
pub(crate) const NORMAL_STAGE_POS: Vec3 = Vec3::new(3.2, 1.95, -0.9);
pub(crate) const STANDARD_HELMET_POS: Vec3 = Vec3::new(6.4, 0.0, -0.95);
pub(crate) const TOON_HELMET_POS: Vec3 = Vec3::new(10.1, 0.0, -0.95);

#[derive(Component)]
pub(crate) struct HeroMesh;

#[derive(Component)]
pub(crate) struct GlossyMesh;

#[derive(Component)]
pub(crate) struct NormalMappedMesh;

#[derive(Component)]
pub(crate) struct LabOverlay;

#[derive(Component)]
pub(crate) struct LabCamera;

#[derive(Component)]
pub(crate) struct SunAnchor;

#[allow(dead_code)]
#[derive(Resource, Clone)]
pub(crate) struct LabAssets {
    pub warm_ramp: Handle<Image>,
    pub cool_ramp: Handle<Image>,
    pub pop_ramp: Handle<Image>,
}

#[allow(dead_code)]
#[derive(Resource, Clone, Copy)]
pub(crate) struct LabEntities {
    pub hero: Entity,
    pub glossy: Entity,
    pub normal_mapped: Entity,
    pub standard_scene_root: Entity,
    pub toon_scene_root: Entity,
    pub camera: Entity,
    pub sun: Entity,
    pub overlay: Entity,
}

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Resource, Clone, Copy)]
pub(crate) struct LabSunRig {
    pub orbit: bool,
    pub translation: Vec3,
    pub target: Vec3,
}

impl Default for LabSunRig {
    fn default() -> Self {
        Self {
            orbit: true,
            translation: Vec3::ZERO,
            target: Vec3::new(2.8, 1.1, -0.9),
        }
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.032, 0.036, 0.05)))
        .init_resource::<LabSunRig>()
        .insert_resource(WinitSettings::continuous())
        .add_plugins((
            common::default_plugins("toon_shader crate-local lab"),
            ToonShaderPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, animate_sun, update_overlay));
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_port(lab_brp_port()));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::ToonShaderLabE2EPlugin);

    install_lab_auto_exit(&mut app);
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let warm_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.26, 0.22, 0.28),
            Color::srgb(0.52, 0.44, 0.34),
            Color::srgb(0.82, 0.7, 0.5),
            Color::srgb(1.0, 0.94, 0.78),
        ],
    );
    let cool_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.2, 0.24, 0.34),
            Color::srgb(0.36, 0.5, 0.7),
            Color::srgb(0.66, 0.82, 0.96),
            Color::srgb(1.0, 1.0, 1.0),
        ],
    );
    let pop_ramp = common::ramp_texture(
        &mut images,
        &[
            Color::srgb(0.22, 0.18, 0.28),
            Color::srgb(0.38, 0.3, 0.7),
            Color::srgb(0.78, 0.5, 0.96),
            Color::srgb(1.0, 0.76, 0.3),
        ],
    );
    let checker = common::checker_texture(&mut images, 8);
    let normal_map = common::normal_map_texture(&mut images, 128, 0.9);

    commands.insert_resource(LabAssets {
        warm_ramp: warm_ramp.clone(),
        cool_ramp,
        pop_ramp,
    });

    let camera = commands
        .spawn((
            Name::new("Lab Camera"),
            LabCamera,
            Camera3d::default(),
            Transform::from_xyz(2.4, 3.8, 12.5).looking_at(Vec3::new(2.8, 1.1, -1.0), Vec3::Y),
        ))
        .id();

    let sun = common::spawn_lighting(&mut commands);
    commands.entity(sun).insert(SunAnchor);
    common::spawn_ground(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
    );
    spawn_plinth(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        "Hero Plinth",
        Vec3::new(HERO_STAGE_POS.x, 0.4, HERO_STAGE_POS.z),
        Color::srgb(0.27, 0.31, 0.39),
    );
    spawn_plinth(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        "Glossy Plinth",
        Vec3::new(GLOSSY_STAGE_POS.x, 0.34, GLOSSY_STAGE_POS.z),
        Color::srgb(0.34, 0.29, 0.26),
    );
    spawn_plinth(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        "Normal Map Plinth",
        Vec3::new(NORMAL_STAGE_POS.x, 0.38, NORMAL_STAGE_POS.z),
        Color::srgb(0.24, 0.3, 0.34),
    );

    let hero = commands
        .spawn((
            Name::new("Hero Mesh"),
            HeroMesh,
            Mesh3d(meshes.add(Sphere::new(1.45).mesh().uv(32, 18))),
            MeshMaterial3d(standard_materials.add(StandardMaterial {
                base_color: Color::srgb(0.88, 0.78, 0.67),
                perceptual_roughness: 0.38,
                ..default()
            })),
            Transform::from_translation(HERO_STAGE_POS),
            DemoSpin {
                axis: Vec3::new(0.0, 1.0, 0.18),
                speed: 0.42,
            },
            ToonExtension::anime_character(),
        ))
        .id();

    let glossy = commands
        .spawn((
            Name::new("Glossy Mesh"),
            GlossyMesh,
            Mesh3d(meshes.add(Torus::new(1.0, 0.34))),
            MeshMaterial3d(standard_materials.add(StandardMaterial {
                base_color: Color::srgb(0.86, 0.22, 0.16),
                perceptual_roughness: 0.18,
                metallic: 0.02,
                ..default()
            })),
            Transform::from_translation(GLOSSY_STAGE_POS)
                .with_rotation(Quat::from_rotation_x(0.92)),
            DemoSpin {
                axis: Vec3::new(0.18, 1.0, 0.08),
                speed: 0.58,
            },
            ToonExtension::glossy_vehicle(),
        ))
        .id();

    let normal_mapped = commands
        .spawn((
            Name::new("Normal Mapped Mesh"),
            NormalMappedMesh,
            Mesh3d(meshes.add(
                Sphere::new(1.25)
                    .mesh()
                    .uv(32, 18)
                    .with_generated_tangents()
                    .expect("UV sphere should generate tangents"),
            )),
            MeshMaterial3d(standard_materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(checker),
                normal_map_texture: Some(normal_map),
                perceptual_roughness: 0.44,
                ..default()
            })),
            Transform::from_translation(NORMAL_STAGE_POS),
            DemoSpin {
                axis: Vec3::new(0.12, 1.0, 0.24),
                speed: 0.34,
            },
            ToonExtension::low_poly_prop(),
        ))
        .id();

    let flight_helmet = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"),
    );
    let standard_scene_root = commands
        .spawn((
            Name::new("Standard Flight Helmet"),
            SceneRoot(flight_helmet.clone()),
            Transform::from_translation(STANDARD_HELMET_POS).with_scale(Vec3::splat(2.3)),
        ))
        .id();

    let toon_scene_root = commands
        .spawn((
            Name::new("Toon Flight Helmet"),
            SceneRoot(flight_helmet),
            Transform::from_translation(TOON_HELMET_POS).with_scale(Vec3::splat(2.3)),
            ToonExtension::low_poly_prop()
                .with_ramp_texture(warm_ramp)
                .with_shadow_floor(0.52)
                .with_shadow_tint(Color::srgb(0.42, 0.46, 0.56))
                .with_rim(
                    saddle_rendering_toon_shader::ToonRim::default()
                        .with_intensity(0.18)
                        .with_threshold(0.5)
                        .with_softness(0.2),
                ),
        ))
        .id();

    let overlay = commands
        .spawn((
            Name::new("Toon Shader Lab Overlay"),
            LabOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: px(14.0),
                top: px(14.0),
                width: px(330.0),
                padding: UiRect::all(px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.04, 0.06, 0.82)),
            Text::new("Toon Shader Lab"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    commands.insert_resource(LabEntities {
        hero,
        glossy,
        normal_mapped,
        standard_scene_root,
        toon_scene_root,
        camera,
        sun,
        overlay,
    });
}

fn spawn_plinth(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    translation: Vec3,
    color: Color,
) {
    commands.spawn((
        Name::new(name.to_owned()),
        Mesh3d(meshes.add(Cuboid::new(2.25, 0.8, 2.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.96,
            reflectance: 0.08,
            ..default()
        })),
        Transform::from_translation(translation),
    ));
}

fn animate_sun(
    time: Res<Time>,
    rig: Res<LabSunRig>,
    mut sun: Query<&mut Transform, With<SunAnchor>>,
) {
    let Ok(mut transform) = sun.single_mut() else {
        return;
    };

    let (translation, target) = if rig.orbit {
        let angle = time.elapsed_secs() * 0.18;
        (
            Vec3::new(angle.cos() * 10.0, 14.0, angle.sin() * 8.5),
            Vec3::new(2.8, 1.1, -0.9),
        )
    } else {
        (rig.translation, rig.target)
    };

    *transform = Transform::from_translation(translation).looking_at(target, Vec3::Y);
}

fn update_overlay(
    diagnostics: Res<ToonShaderDiagnostics>,
    entities: Res<LabEntities>,
    mut overlay: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok(mut text) = overlay.single_mut() else {
        return;
    };

    let mut body = String::new();
    let _ = writeln!(&mut body, "Toon Shader Lab");
    let _ = writeln!(&mut body, "runtime active: {}", diagnostics.runtime_active);
    let _ = writeln!(
        &mut body,
        "managed direct meshes: {}",
        diagnostics.managed_direct_entities
    );
    let _ = writeln!(
        &mut body,
        "managed scene meshes: {}",
        diagnostics.managed_scene_entities
    );
    let _ = writeln!(&mut body, "scene roots: {}", diagnostics.scene_roots);
    let _ = writeln!(&mut body, "toon material assets: {}", diagnostics.toon_material_assets);
    let _ = writeln!(
        &mut body,
        "ramp/spec/rim: {}/{}/{}",
        diagnostics.ramp_materials,
        diagnostics.specular_enabled_materials,
        diagnostics.rim_enabled_materials
    );
    let _ = writeln!(&mut body);
    let _ = writeln!(&mut body, "hero: {:?}", entities.hero);
    let _ = writeln!(&mut body, "glossy: {:?}", entities.glossy);
    let _ = writeln!(&mut body, "normal map: {:?}", entities.normal_mapped);
    let _ = writeln!(&mut body, "toon scene: {:?}", entities.toon_scene_root);

    text.0 = body;
}

fn install_lab_auto_exit(app: &mut App) {
    let Some(seconds) = std::env::var(LAB_EXIT_ENV)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|seconds| *seconds > 0.0)
    else {
        return;
    };

    app.insert_resource(AutoExitAfter(Timer::from_seconds(seconds, TimerMode::Once)));
    app.add_systems(Update, auto_exit_after);
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_BRP_PORT)
}

#[cfg(not(feature = "dev"))]
fn lab_brp_port() -> u16 {
    DEFAULT_BRP_PORT
}
