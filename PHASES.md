# Mork — Phase Plan

## Phase 1: Arena Combat

**Goal**: Dark Souls combat feel in a contained arena with one enemy.

### Steps

1. `cargo init` + `Cargo.toml` with all dependencies
2. Minimal Bevy app (window, default plugins)
3. Ground plane + capsule player with WASD movement
4. Third-person camera (free orbit + lock-on)
5. Second capsule (enemy) with idle state
6. Input manager mapping (keyboard + Xbox gamepad)
7. Light attack + enemy hit reaction
8. Heavy attack
9. Block with stamina drain
10. Roll with iframes
11. Stamina bar (egui)
12. Poise/stagger system
13. Enemy AI state machine (idle/chase/attack)
14. Combat juice (hit flash, screen shake, particles)
15. Audio (attack whoosh, hit, block clang, dodge)

### Key Decisions

- Hand-built arena (Blender for static geometry)
- Capsule characters (Mixamo animations)
- Simple state machine AI
- Pure logic extraction for testable combat math
- No procedural generation, no roguelite mechanics
- No save system

### Completion Criteria

- Player can move, dodge, attack (light + heavy), block
- Enemy chases, attacks, reacts to hits
- Combat feels responsive at 60 FPS
- Stamina and poise systems functional
- Lock-on camera works
- Xbox controller fully supported

---

## Phase 2: Procedural Forest + Roguelite Loop

**Goal**: Procedurally generated forest with roguelite run structure.

### Key Decisions

- Linear descent, 3 zones per run (inspired by Returnal + Binding of Isaac)
- Hybrid procgen: roaming forest sections + curated encounter clearings
- Room-based clearings with controlled enemy spawns
- Meta-progression: currency (lose 50% on death) + blueprints (weighted random by zone)
- Abstract egui menu as hub between runs
- Shortcut unlocks: reach a zone → start from there in future runs
- Weapons only (2-3 types), found per-run, lost on death
- 2-3 enemy types total (phase 1 reused + 1 new + zone 3 boss)
- Zone 3 boss: two-phase fight
- Difficulty scaling via enemy composition (new types in deeper zones)
- Procedural generation approach: deferred until combat is proven
- Instanced tree rendering, built-in distance fog
- Real-time directional + point lights

### Steps (rough)

1. Procedural forest generation (terrain + tree placement)
2. Encounter clearing generation
3. Zone progression system (3 zones)
4. Weapon pickups/equipment system
5. Currency + blueprint meta-progression
6. Hub menu (egui)
7. Shortcut unlock system
8. New enemy type for zone 2
9. Zone 3 boss (two-phase)
10. Death → hub loop
11. Forest atmosphere (fog, lighting, ambient audio)

### Completion Criteria

- Full run: zone 1 forest → zone 2 forest → zone 3 boss → win/die
- Meta-progression: currency and blueprints persist across runs
- Shortcut unlocks work
- 2-3 weapon types findable per run
- Distinct enemy types per zone
- Boss fight feels like a boss

---

## Phase 3+: Incremental Expansion

All three tracks, prioritized by what's weakest:

### Content

- More zones
- More enemy types
- More weapon types
- More blueprint variety

### Systems

- NPC vendors in hub
- Crafting from blueprints
- Status effects (poison, bleed)
- Weapon upgrades within a run
- Armor system with equip load
- 3D hub zone (Blender learning project)
- Save system (serde + file I/O)

### Polish

- Hitstop (freeze frames on impact)
- Time slowdown on parry/kill
- Boss cinematics
- Environmental storytelling
- Ambient audio (wind, creaking trees)
- Ground mist particles
- Volumetric fog (if performance allows)
- Billboard impostors for distant trees (if performance requires)
