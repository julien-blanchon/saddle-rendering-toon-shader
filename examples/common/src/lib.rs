use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::asset::{AssetPlugin, RenderAssetUsages};
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::winit::WinitSettings;

pub const AUTO_EXIT_ENV: &str = "TOON_SHADER_AUTO_EXIT_SECONDS";

#[derive(Component)]
pub struct DemoSpin {
    pub axis: Vec3,
    pub speed: f32,
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
    if manifest_dir.ends_with("lab") {
        manifest_dir
            .parent()
            .expect("lab crate should sit under examples/")
            .join("assets")
    } else {
        manifest_dir.join("examples").join("assets")
    }
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
