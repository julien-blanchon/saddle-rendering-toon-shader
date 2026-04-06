# Saddle Rendering Toon Shader

Reusable toon and cel shading for Bevy PBR scenes. The crate keeps Bevy's `StandardMaterial` workflow, supports per-material diffuse banding or ramp textures, adds optional specular and rim bands, and includes a scene-replacement path for imported glTF content.

## Quick Start

```toml
saddle-rendering-toon-shader = { git = "https://github.com/julien-blanchon/saddle-rendering-toon-shader" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_rendering_toon_shader::{ToonExtension, ToonRim, ToonShaderPlugin, ToonSpecular};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ToonShaderPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<saddle_rendering_toon_shader::ToonMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let material = materials.add(
        ToonExtension::banded(2)
            .with_shadow_response(0.12, Color::srgb(0.43, 0.48, 0.70))
            .with_specular(
                ToonSpecular::default()
                    .with_intensity(0.9)
                    .with_width(0.42)
                    .with_threshold(0.54),
            )
            .with_rim(
                ToonRim::default()
                    .with_intensity(0.28)
                    .with_threshold(0.55)
                    .with_softness(0.18),
            )
            .material(StandardMaterial {
                base_color: Color::srgb(0.92, 0.82, 0.72),
                perceptual_roughness: 0.45,
                ..default()
            }),
    );

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.2).mesh().ico(5).unwrap())),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 1.25, 0.0),
    ));
}
```

## Scene Replacement

Attach `ToonExtension` to a `SceneRoot` to replace descendant `StandardMaterial` handles once the imported scene entities exist:

```rust,no_run
use bevy::prelude::*;
use saddle_rendering_toon_shader::{ToonExtension, ToonSpecular};

fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(asset_server.load("models/hero.glb#Scene0")),
        Transform::from_scale(Vec3::splat(1.4)),
        ToonExtension::banded(3)
            .with_band_softness(0.08)
            .with_shadow_response(0.18, Color::srgb(0.32, 0.36, 0.46))
            .with_specular(ToonSpecular::disabled()),
    ));
}
```

Attach `ToonExtension` directly to a mesh entity with `MeshMaterial3d<StandardMaterial>` for the same replacement flow on non-scene content.

## Public API

| Type | Purpose |
|------|---------|
| `ToonShaderPlugin` | Registers the material extension, replacement systems, diagnostics, and embedded shader |
| `ToonShaderSystems::ReplaceSceneMaterials` | Converts direct or scene-descendant `StandardMaterial` handles into toon materials |
| `ToonShaderSystems::SyncRuntimeParameters` | Pushes changed `ToonExtension` component values into already-converted toon materials |
| `ToonMaterial` | `ExtendedMaterial<StandardMaterial, ToonExtension>` alias |
| `ToonExtension` | Per-material toon parameters and direct/scene assignment component |
| `ToonDiffuseMode` | Choose stepped band mode or ramp-texture mode |
| `ToonSpecular` | Thresholded specular band controls |
| `ToonRim` | Fresnel-style rim controls |
| `ToonShaderDiagnostics` | Runtime counters for BRP, overlays, and E2E assertions |

## Material Model

- Diffuse stylization operates on the lit result of Bevy PBR with shadows included, then remaps that intensity into stepped bands or a sampled ramp texture.
- Specular is estimated from a second lighting probe and thresholded into a separate artist-controlled band.
- Rim lighting uses the final shaded normal, so normal maps still affect the silhouette highlight direction.
- Forward rendering produces the full stylized look. Deferred rendering still compiles, but the deferred branch writes compatible PBR data because the post-lighting probes required for banding are unavailable there.

## Neutral Builders

The core crate intentionally exposes low-level tuning builders instead of named style presets:

- `ToonExtension::banded(count)` and `ToonExtension::ramped(handle)` choose the diffuse mode
- `with_band_profile`, `with_shadow_response`, `without_specular`, and `without_rim` cover common tuning bundles
- `ToonSpecular::disabled()` and `ToonRim::disabled()` make opt-out configuration explicit

Example-only named looks now live in [`examples/common/src/sample_looks.rs`](examples/common/src/sample_looks.rs).

## Examples

All examples use `saddle-pane` for live parameter editing and include on-screen instructions.

| Example | Purpose | Run |
|---------|---------|-----|
| `basic` | Side-by-side PBR vs Toon comparison with live band/shadow/rim controls | `cargo run -p saddle-rendering-toon-shader-example-basic` |
| `ramps` | Compare discrete banding with artist-authored ramp textures | `cargo run -p saddle-rendering-toon-shader-example-ramps` |
| `rim_specular` | Three spheres with different rim/specular styles and 6 live sliders | `cargo run -p saddle-rendering-toon-shader-example-rim-specular` |
| `gltf_replace` | Side-by-side original PBR vs toon-shaded FlightHelmet glTF scene | `cargo run -p saddle-rendering-toon-shader-example-gltf-replace` |
| `showcase` | Cycle through 6 example-only sample looks applied to 5 different shapes | `cargo run -p saddle-rendering-toon-shader-example-showcase` |
| `outline_optional` | Show optional integration with `saddle-rendering-outline` plus live rim/outline controls | `cargo run -p saddle-rendering-toon-shader-example-outline-optional` |

## Crate-Local Lab

The richer verification target lives at `shared/rendering/saddle-rendering-toon-shader/examples/lab`:

```bash
cargo run -p saddle-rendering-toon-shader-lab
```

Focused E2E scenarios:

```bash
cargo run -p saddle-rendering-toon-shader-lab --features e2e -- toon_shader_smoke
cargo run -p saddle-rendering-toon-shader-lab --features e2e -- toon_shader_gltf_replace
cargo run -p saddle-rendering-toon-shader-lab --features e2e -- toon_shader_rim_specular
```

BRP targets in the lab:

- Resource: `saddle_rendering_toon_shader::components::ToonShaderDiagnostics`
- Component: `saddle_rendering_toon_shader::material::ToonExtension`
- Scene asset: crate-local `examples/assets/models/FlightHelmet/`

Example commands:

```bash
BRP_EXTRAS_PORT=15744 cargo run -p saddle-rendering-toon-shader-lab
BRP_PORT=15744 uv run --active --project .codex/skills/bevy-brp/script brp \
  resource get 'saddle_rendering_toon_shader::components::ToonShaderDiagnostics'
BRP_PORT=15744 uv run --active --project .codex/skills/bevy-brp/script brp \
  extras screenshot /tmp/toon_shader_lab.png
```

## Limitations

- The full toon look is a forward-rendering feature. If a material or project opts into deferred rendering, the crate now respects that choice and falls back to the compatible deferred PBR branch instead of forcing the material back to forward.
- Ramp textures should be authored with a predictable sampler setup. The examples generate ramps in code and use nearest sampling to keep transitions crisp.
- Alpha-clipped and transparent materials work through `StandardMaterial`, but every project should validate foliage, hair cards, and transmissive surfaces with its own art assets.
- The crate does not include outline rendering. Thick outlines need different tradeoffs (screen-space vs inverted-hull) and should be handled separately.
- The crate-local lab now uses a textured `FlightHelmet` glTF and completion-aware scene replacement assertions, but the direct-mesh showcase still depends heavily on light direction and should be treated as a qualitative check rather than a golden snapshot.

More detail lives in [architecture.md](docs/architecture.md) and [configuration.md](docs/configuration.md).
