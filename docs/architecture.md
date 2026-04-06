# Architecture

## Why `ExtendedMaterial`

`saddle-rendering-toon-shader` is built on `ExtendedMaterial<StandardMaterial, ToonExtension>` instead of a full custom `Material`.

That choice keeps the parts of Bevy PBR that make shared stylized materials usable across many projects:

- `StandardMaterial` textures, normal maps, alpha modes, and glTF import compatibility
- Bevy shadow casting and receiving
- built-in fog, tonemapping, and post-lighting pipeline integration on the forward path
- familiar material authoring ergonomics for users already shipping PBR scenes

The crate only replaces the fragment shader stage where Bevy makes the lit result available.

## Shader Data Flow

Forward rendering uses three steps:

1. Build the usual `PbrInput` with `pbr_input_from_standard_material`.
2. Run two probe lighting passes with the base color forced to white:
   - a diffuse-only probe with specular features disabled
   - a full probe that keeps specular enabled
3. Derive stylized outputs from those probes:
   - diffuse probe intensity becomes stepped bands or a ramp texture lookup
   - the difference between full and diffuse probes becomes the thresholded specular band
   - rim lighting is added from the final shaded normal and view direction

Because the diffuse probe still goes through Bevy's shadow samplers, quantization respects light plus shadow contribution instead of ignoring shadows.

## Forward And Deferred Notes

Bevy's deferred prepass only exposes material inputs, not the final lit result. Toon banding in this crate depends on post-lighting data, so the stylized effect is implemented on the forward path.

The shader still provides a deferred branch so materials compile cleanly in mixed projects, but that branch writes the normal deferred output without stylization. Converted materials therefore preserve the base `StandardMaterial::opaque_render_method` instead of overriding the project's render-path choice.

## Replacement Lifecycle

`ToonExtension` doubles as both:

- the material extension stored inside `ToonMaterial`
- the authored component attached to direct meshes or `SceneRoot` entities

Replacement happens in `ToonShaderSystems::ReplaceSceneMaterials`:

1. Direct entities with `MeshMaterial3d<StandardMaterial>` and `ToonExtension` convert immediately.
2. `SceneRoot` entities with `ToonExtension` scan descendants every frame.
3. As soon as descendant mesh entities exist, their `StandardMaterial` handles are replaced with `ToonMaterial`.
4. Within a single scene root, repeated descendants that shared the same original `StandardMaterial` reuse one toon material handle.
5. Once conversion is complete, the root receives an internal completion marker, so later updates stop rescanning the descendant tree until the `SceneRoot` changes again.

This polling approach keeps the system testable without needing a real `SceneSpawner` setup in unit tests, still works when scene children appear asynchronously, and avoids frame-over-frame scan cost once a root has fully converted.

## Runtime Sync

`ToonShaderSystems::SyncRuntimeParameters` listens for `Changed<ToonExtension>` and pushes updated extension values into already-converted materials.

- Direct entities update their own toon material handle.
- Scene roots update descendant toon materials unless a descendant carries its own local `ToonExtension` override.

That override rule makes root-level scene styling composable:

- root `ToonExtension` defines the scene-wide default
- descendant `ToonExtension` opts a child mesh into a different toon profile

## Diagnostics

`ToonShaderDiagnostics` is updated every frame and intentionally stays simple:

- runtime active flag
- managed direct entity count
- managed scene entity count
- scene root count
- toon material asset count
- count of ramp, rim, and specular-enabled materials

The diagnostics resource exists for BRP inspection, overlay text in the lab, and E2E assertions.

## Outline Scope Decision

Outline support is intentionally not part of this crate. Thick outlines need different tradeoffs than shaded-band materials, and screen-space and inverted-hull outlines deserve their own performance and topology rules.
