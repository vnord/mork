# Mork — Design Document

A 3D dark fantasy roguelite with Souls-like combat and procedurally generated forests. Built in Rust
with Bevy to learn Rust, gamedev, and asset creation.

## Tech Stack

| Component    | Choice                 | Version |
| ------------ | ---------------------- | ------- |
| Engine       | Bevy                   | 0.18    |
| Physics      | Rapier (bevy_rapier3d) | 0.33    |
| Input        | leafwing-input-manager | 0.20    |
| UI/Debug     | bevy_egui              | 0.39    |
| Audio        | bevy_kira_audio        | 0.25    |
| 3D Modeling  | Blender                | latest  |
| Animations   | Mixamo (.fbx)          | —       |
| Audio Assets | freesound.org          | —       |

## Target Platform

- MacBook Pro M2 (Apple Silicon, Metal GPU)
- 60 FPS target
- Xbox controller (primary), keyboard (secondary)

## Visual Style

Dark stylized — dark atmosphere with simplified/stylized geometry and textures. Fog, lighting, and
color palette carry the mood. Not photorealistic PBR.

## Project Structure

```
mork/
├── src/
│   ├── main.rs
│   ├── plugins/        # Bevy plugins (combat, enemy, camera, etc.)
│   ├── components/     # ECS component definitions
│   ├── resources/      # ECS resource definitions
│   └── systems/        # Shared system logic
├── assets/
│   ├── models/
│   ├── animations/
│   ├── sounds/
│   └── textures/
└── Cargo.toml
```

## Architecture

- **Bevy plugins** — each major system is its own plugin
- **Cargo feature flags** — toggle systems for fast iteration (`combat`, `camera`, `input`, `audio`,
  `debug_ui`, `procgen`, `roguelite`)
- **Trunk-based development** — all work on main, features toggled via Cargo features
- **Logic extraction** — combat math and state machines as pure functions, unit-testable, wired into
  ECS

## Combat Design

Core Souls-like mechanics:

- Light attack, heavy attack
- Block with stamina drain
- Roll with iframes
- Lock-on camera system
- Stamina management
- Poise/stagger system

Input: keyboard + Xbox gamepad via leafwing-input-manager (with input buffering).

## Enemy AI

Simple state machine: `Idle → Chase → Attack(cooldown) → Idle`. One enemy type in phase 1.

## Camera

Free orbit when roaming, snap-to-locked-enemy in combat. State machine switching between modes.

## Juice & Feedback

- Hit flash (material color swap for 2 frames)
- Screen shake on heavy hits
- Impact particles
- Stamina bar pulse when low
- Distinct audio per action (whoosh/hit/block clang/dodge)

## Asset Pipeline

- Characters: capsule primitives in Bevy (replaced later)
- Arena geometry: Blender (hand-built)
- Animations: Mixamo .fbx, retargeted to custom skeletons later
- Audio: freesound.org placeholders, custom later

## Linting

- `cargo fmt` + `cargo clippy` with pedantic and nursery lints
- Enforced via prek

## Build

- `cargo run` initially, `cargo watch -x run` when compile times warrant it
- Dev profile: `opt-level = 1` for workspace, `opt-level = 3` for dependencies

## Learning Approach

- Primary: AI coding assistant for patterns and boilerplate
- Secondary: Bevy examples, Bevy Discord, docs.rs
- Always verify AI-generated Bevy API calls against docs.rs (stale training data)
