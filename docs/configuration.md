# Configuration

## `ToonExtension`

| Field | Type | Default | Expected Range | Effect | Notes |
|------|------|---------|----------------|--------|-------|
| `diffuse_mode` | `ToonDiffuseMode` | `Bands` | `Bands` or `RampTexture` | Chooses stepped math bands or ramp texture sampling | `RampTexture` falls back to `Bands` if no texture handle is present |
| `band_count` | `u32` | `2` | `2..=8` | Number of diffuse light bands in `Bands` mode | Values outside range are clamped in the GPU uniform conversion |
| `band_softness` | `f32` | `0.12` | `0.0..=1.0` | Smooths transitions between bands | `0.0` is hard cel shading; larger values soften the contour |
| `shadow_floor` | `f32` | `0.16` | `0.0..=1.0` | Lifts shadowed regions toward the lit band | Good for readability at distance or in low-ambient scenes |
| `shadow_tint` | `Color` | `srgb(0.22, 0.24, 0.30)` | artistic | Tints the darkest diffuse band | Neutral dark blue-grey works well as a reusable default |
| `light_wrap` | `f32` | `0.0` | `-0.5..=0.5` practical | Shifts the diffuse threshold before quantization | Positive values broaden the lit side; negative values tighten it |
| `specular` | `ToonSpecular` | subtle highlight | see below | Controls the thresholded specular band | Set `intensity = 0.0` to disable |
| `rim` | `ToonRim` | disabled | see below | Controls the Fresnel rim band | Set `intensity = 0.0` to disable |
| `ramp_texture` | `Option<Handle<Image>>` | `None` | 1D or 2D ramp strip | Artist-authored LUT sampled in `RampTexture` mode | The examples generate ramps with nearest sampling for clean bands |

## `ToonSpecular`

| Field | Type | Default | Expected Range | Effect | Notes |
|------|------|---------|----------------|--------|-------|
| `color` | `Color` | `WHITE` | artistic | Final tint for the specular band | Multiplies the normalized specular probe tint |
| `threshold` | `f32` | `0.58` | `0.0..=1.0` | Where the band begins | Lower values widen the highlight |
| `softness` | `f32` | `0.08` | `0.0..=1.0` | Softens the edge of the highlight band | A small non-zero value reduces shimmering |
| `intensity` | `f32` | `0.12` | `0.0..=2.0` practical | Strength of the specular contribution | `0.0` disables specular |
| `width` | `f32` | `0.22` | `0.0..=1.0` | Broadens or tightens the probe before thresholding | Higher values make glossy bands easier to hit |

## `ToonRim`

| Field | Type | Default | Expected Range | Effect | Notes |
|------|------|---------|----------------|--------|-------|
| `color` | `Color` | `srgb(1.0, 0.97, 0.90)` | artistic | Final rim tint | Warm off-white stays flexible across art directions |
| `threshold` | `f32` | `0.62` | `0.0..=1.0` | Grazing-angle threshold for the rim | Lower values create a broader rim |
| `softness` | `f32` | `0.12` | `0.0..=1.0` | Softens the rim edge | Slight softness reduces aliasing on moving silhouettes |
| `intensity` | `f32` | `0.0` | `0.0..=2.0` practical | Rim brightness | `0.0` disables rim light |
| `lit_side_only` | `bool` | `true` | `true` or `false` | Masks the rim by the diffuse visibility term | Useful for anime-style lit-side rims |

## Neutral Builders

| Builder | Purpose | Notes |
|------|------|------|
| `ToonExtension::banded(count)` | Start from stepped diffuse shading | Sets `diffuse_mode = Bands` |
| `ToonExtension::ramped(handle)` | Start from ramp-texture diffuse shading | Equivalent to `default().with_ramp_texture(handle)` |
| `ToonExtension::with_band_profile(count, softness)` | Tune band count and edge softness together | Keeps `Bands` mode active |
| `ToonExtension::with_shadow_response(floor, tint)` | Tune shadow lift and shadow tint together | Helpful for authoring reusable material defaults |
| `ToonExtension::without_specular()` | Disable the specular band at the extension level | Equivalent to `with_specular(ToonSpecular::disabled())` |
| `ToonExtension::without_rim()` | Disable rim lighting at the extension level | Equivalent to `with_rim(ToonRim::disabled())` |
| `ToonSpecular::disabled()` | Build an explicit no-specular config | `intensity = 0.0` |
| `ToonRim::disabled()` | Build an explicit no-rim config | `intensity = 0.0` |

Named artistic looks are intentionally kept out of the core crate. The example workspace rebuilds sample looks in `examples/common/src/sample_looks.rs` using the builders above.

## Performance Notes

- The shader performs two PBR probe evaluations on the forward path to separate diffuse and specular behavior.
- Ramp textures do not add extra draw passes, only one extra texture sample path in the fragment shader.
- Scene replacement scans descendants while a tagged root is unresolved, then stops rescanning once the root is marked complete.
- Converted materials keep whatever `StandardMaterial::opaque_render_method` the user already chose, so deferred-capable projects can preserve their render-path configuration.
- Outline composition is kept out of the crate runtime to avoid adding another pass or helper mesh system to every consumer.
