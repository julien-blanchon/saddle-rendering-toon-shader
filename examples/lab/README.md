# `toon_shader_lab`

Crate-local runtime verification app for `toon_shader`.

## Purpose

- compare direct toon materials against imported scene replacement
- exercise ramp textures, specular bands, rim lighting, and normal maps in one scene
- provide BRP and E2E-friendly diagnostics for repeatable verification
- keep visual verification inside the shared crate instead of pushing lab code into workspace sandboxes

## Run

```bash
cargo run -p toon_shader_lab
```

Timed auto-exit:

```bash
TOON_SHADER_LAB_EXIT_AFTER_SECONDS=2 cargo run -p toon_shader_lab
```

## E2E

```bash
cargo run -p toon_shader_lab --features e2e -- toon_shader_smoke
cargo run -p toon_shader_lab --features e2e -- toon_shader_gltf_replace
cargo run -p toon_shader_lab --features e2e -- toon_shader_rim_specular
```

## BRP

```bash
BRP_EXTRAS_PORT=15744 cargo run -p toon_shader_lab
BRP_PORT=15744 uv run --active --project .codex/skills/bevy-brp/script brp \
  resource get 'toon_shader::components::ToonShaderDiagnostics'
BRP_PORT=15744 uv run --active --project .codex/skills/bevy-brp/script brp \
  extras screenshot /tmp/toon_shader_lab.png
```

## Assets

The lab uses the crate-local `examples/assets/models/FlightHelmet/` copy sourced from the local Bevy example assets. The direct-mesh lane, plinths, ramps, ground, and normal maps are generated in code.
