use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::asset::{AssetPlugin, RenderAssetUsages};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::winit::WinitSettings;

pub use saddle_pane;
pub use saddle_pane::prelude::*;

/// Returns all plugins required for saddle-pane to work.
/// Usage: `app.add_plugins(common::pane_plugins())`
pub fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

pub const AUTO_EXIT_ENV: &str = "TOON_SHADER_AUTO_EXIT_SECONDS";

#[derive(Component)]
pub struct DemoSpin {
    pub axis: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct OrbitCamera {
    pub center: Vec3,
    pub radius: f32,
    pub speed: f32,
    pub height: f32,
}

#[derive(Resource)]
struct AutoExitAfter(Timer);

pub fn default_plugins(title: &str) -> bevy::app::PluginGroupBuilder {
    DefaultPlugins
        .set(AssetPlugin {
            file_path: asset_root().display().to_string(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: title.into(),
                resolution: (1520, 920).into(),
                ..default()
            }),
            ..default()
        })
}

pub fn install_auto_exit(app: &mut App) {
    if let Some(duration) = auto_exit_seconds() {
        app.insert_resource(WinitSettings::continuous());
        app.insert_resource(AutoExitAfter(Timer::from_seconds(
            duration,
            TimerMode::Once,
        )));
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn spin(time: Res<Time>, mut spinning: Query<(&DemoSpin, &mut Transform)>) {
    for (spin, mut transform) in &mut spinning {
        transform.rotate(Quat::from_axis_angle(
            spin.axis.normalize_or_zero(),
            spin.speed * time.delta_secs(),
        ));
    }
}

pub fn orbit_camera(time: Res<Time>, mut cameras: Query<(&OrbitCamera, &mut Transform)>) {
    for (orbit, mut transform) in &mut cameras {
        let angle = time.elapsed_secs() * orbit.speed;
        let pos = orbit.center
            + Vec3::new(
                angle.cos() * orbit.radius,
                orbit.height,
                angle.sin() * orbit.radius,
            );
        *transform = Transform::from_translation(pos).looking_at(orbit.center, Vec3::Y);
    }
}

pub fn spawn_camera(commands: &mut Commands, translation: Vec3, target: Vec3) -> Entity {
    commands
        .spawn((
            Name::new("Demo Camera"),
            Camera3d::default(),
            Msaa::Sample4,
            Transform::from_translation(translation).looking_at(target, Vec3::Y),
        ))
        .id()
}

pub fn spawn_lighting(commands: &mut Commands) -> Entity {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.86, 0.9, 1.0),
        brightness: 18.0,
        ..default()
    });

    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 2_200.0,
            range: 22.0,
            shadows_enabled: false,
            color: Color::srgb(1.0, 0.82, 0.74),
            ..default()
        },
        Transform::from_xyz(-4.0, 3.5, 8.0),
    ));

    commands
        .spawn((
            Name::new("Sun Light"),
            DirectionalLight {
                illuminance: 22_000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(10.0, 16.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .id()
}

pub fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Entity {
    let albedo = checker_texture(images, 8);
    commands
        .spawn((
            Name::new("Ground"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(26.0, 26.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(albedo),
                perceptual_roughness: 0.95,
                reflectance: 0.08,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id()
}

/// Spawn a rich scene with multiple primitive objects arranged attractively.
/// Returns the entity IDs of all spawned meshes.
pub fn spawn_showcase_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Entity> {
    vec![
        // Central sphere
        commands
            .spawn((
                Name::new("Central Sphere"),
                Mesh3d(meshes.add(Sphere::new(1.2).mesh().uv(48, 24))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.92, 0.78, 0.65),
                    perceptual_roughness: 0.35,
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.3, 0.0),
                DemoSpin {
                    axis: Vec3::new(0.0, 1.0, 0.15),
                    speed: 0.3,
                },
            ))
            .id(),
        // Torus on the left
        commands
            .spawn((
                Name::new("Left Torus"),
                Mesh3d(meshes.add(Torus::new(0.85, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.85, 0.28, 0.22),
                    perceptual_roughness: 0.25,
                    metallic: 0.02,
                    ..default()
                })),
                Transform::from_xyz(-3.2, 1.2, -0.4).with_rotation(Quat::from_rotation_x(0.8)),
                DemoSpin {
                    axis: Vec3::new(0.2, 1.0, 0.1),
                    speed: 0.45,
                },
            ))
            .id(),
        // Cube on the right
        commands
            .spawn((
                Name::new("Right Cube"),
                Mesh3d(meshes.add(Cuboid::new(1.4, 1.4, 1.4))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.38, 0.62, 0.82),
                    perceptual_roughness: 0.45,
                    ..default()
                })),
                Transform::from_xyz(3.2, 0.85, -0.2).with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0.3,
                    0.6,
                    0.15,
                )),
                DemoSpin {
                    axis: Vec3::new(0.1, 1.0, 0.2),
                    speed: 0.25,
                },
            ))
            .id(),
        // Capsule behind
        commands
            .spawn((
                Name::new("Back Capsule"),
                Mesh3d(meshes.add(Capsule3d::new(0.5, 1.6))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.72, 0.82, 0.48),
                    perceptual_roughness: 0.55,
                    ..default()
                })),
                Transform::from_xyz(1.0, 1.35, -3.0).with_rotation(Quat::from_rotation_z(0.25)),
                DemoSpin {
                    axis: Vec3::Y,
                    speed: 0.35,
                },
            ))
            .id(),
        // Cylinder on the far left
        commands
            .spawn((
                Name::new("Left Cylinder"),
                Mesh3d(meshes.add(Cylinder::new(0.6, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.88, 0.68, 0.42),
                    perceptual_roughness: 0.6,
                    ..default()
                })),
                Transform::from_xyz(-1.8, 1.0, -2.5),
                DemoSpin {
                    axis: Vec3::Y,
                    speed: 0.2,
                },
            ))
            .id(),
    ]
}

/// Spawn on-screen instructions text in the bottom-left corner.
pub fn spawn_instructions(commands: &mut Commands, text: &str) -> Entity {
    commands
        .spawn((
            Name::new("Instructions Overlay"),
            Node {
                position_type: PositionType::Absolute,
                left: px(16.0),
                bottom: px(16.0),
                max_width: px(420.0),
                padding: UiRect::all(px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            Text::new(text.to_owned()),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        ))
        .id()
}

pub fn checker_texture(images: &mut Assets<Image>, size: u32) -> Handle<Image> {
    let mut image = blank_image(size, size, TextureFormat::Rgba8UnormSrgb);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });

    for y in 0..size {
        for x in 0..size {
            let dark = ((x / 2) + (y / 2)) % 2 == 0;
            let color = if dark {
                Color::srgb(0.18, 0.2, 0.24)
            } else {
                Color::srgb(0.68, 0.71, 0.76)
            };
            let _ = image.set_color_at(x, y, color);
        }
    }

    images.add(image)
}

pub fn ramp_texture(images: &mut Assets<Image>, colors: &[Color]) -> Handle<Image> {
    let width = colors.len().max(1) as u32;
    let mut image = blank_image(width, 1, TextureFormat::Rgba8UnormSrgb);
    image.sampler = ImageSampler::nearest();

    for (index, color) in colors.iter().enumerate() {
        let _ = image.set_color_at(index as u32, 0, *color);
    }

    images.add(image)
}

pub fn normal_map_texture(images: &mut Assets<Image>, size: u32, strength: f32) -> Handle<Image> {
    let mut image = blank_image(size, size, TextureFormat::Rgba8Unorm);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });

    for y in 0..size {
        for x in 0..size {
            let fx = x as f32 / size as f32;
            let fy = y as f32 / size as f32;
            let height_x = ((fx * std::f32::consts::TAU * 4.0).sin()
                + (fy * std::f32::consts::TAU * 2.5).cos())
                * 0.5;
            let height_y = ((fx * std::f32::consts::TAU * 3.0).cos()
                + (fy * std::f32::consts::TAU * 5.0).sin())
                * 0.5;
            let normal = Vec3::new(-height_x * strength, -height_y * strength, 1.0).normalize();
            let _ = image.set_color_at(
                x,
                y,
                Color::linear_rgba(
                    normal.x * 0.5 + 0.5,
                    normal.y * 0.5 + 0.5,
                    normal.z * 0.5 + 0.5,
                    1.0,
                ),
            );
        }
    }

    images.add(image)
}

fn blank_image(width: u32, height: u32, format: TextureFormat) -> Image {
    Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        format,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

fn asset_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // All example crates sit under examples/<name>/ (or examples/lab/).
    // The shared assets live at examples/assets/.
    manifest_dir
        .parent()
        .expect("example crate should sit under examples/")
        .join("assets")
}

fn auto_exit_seconds() -> Option<f32> {
    std::env::var(AUTO_EXIT_ENV)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|seconds| *seconds > 0.0)
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
