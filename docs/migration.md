# Migration

## From the initial preset-based API

The crate no longer exposes named style constructors on `ToonExtension`.

Replace calls like:

- `ToonExtension::anime_character()`
- `ToonExtension::low_poly_prop()`
- `ToonExtension::glossy_vehicle()`
- `ToonExtension::wind_waker()`
- `ToonExtension::borderlands()`
- `ToonExtension::flat_cel()`

with the neutral builders:

- `ToonExtension::banded(count)` or `ToonExtension::ramped(handle)`
- `with_band_profile(...)`
- `with_shadow_response(...)`
- `with_specular(...)` or `without_specular()`
- `with_rim(...)` or `without_rim()`

If you still want the old named looks for demos, the example workspace now rebuilds them in `examples/common/src/sample_looks.rs`.

## Render-path behavior

`ToonExtension::material(...)` and scene replacement no longer force `OpaqueRendererMethod::Forward`.

- Forward materials still get the full stylized banding path.
- Deferred materials now stay deferred and use the compatibility branch that writes normal deferred PBR output.

Projects that always want the stylized look should keep their base material render method on forward, or leave it at `Auto` while using Bevy's forward default.
